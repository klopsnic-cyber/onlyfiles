// Paralleler Verzeichnis-Scan. Ergebnis ist ein Baum, dessen Kinder nach Größe absteigend
// sortiert sind; ein Knoten wird über seinen Indexpfad ab der Wurzel adressiert ([0, 3, 1]).
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::Relaxed};

pub struct Node {
    pub name: Box<str>,
    pub size: u64,
    pub files: u64,
    pub dir: bool,
    pub children: Vec<Node>,
}

#[derive(Default)]
pub struct Progress {
    pub files: AtomicU64,
    pub bytes: AtomicU64,
    pub denied: AtomicU64,
    pub cancel: AtomicBool,
    pub current: Mutex<String>,
}

struct Ctx<'a> {
    p: &'a Progress,
    devs: Vec<u64>,
    skip: Vec<PathBuf>,
    hardlinks: Mutex<HashSet<(u64, u64)>>,
}

pub fn scan(root: &Path, p: &Progress) -> Node {
    let mut devs = vec![fs::metadata(root).map(|m| m.dev()).unwrap_or(0)];
    let mut skip = vec![];
    if root == Path::new("/") {
        // Ab Catalina liegt „Macintosh HD“ auf zwei Volumes; die Daten-Seite ist über
        // Firmlinks (/Users, /Applications …) schon unter / sichtbar. /System/Volumes
        // würde sie doppelt zählen.
        devs.extend(fs::metadata("/System/Volumes/Data").map(|m| m.dev()));
        skip.push(PathBuf::from("/System/Volumes"));
    }
    let ctx = Ctx { p, devs, skip, hardlinks: Mutex::default() };
    let name = root.file_name().map(|n| n.to_string_lossy().into()).unwrap_or_else(|| root.to_string_lossy().into());
    scan_dir(root, name, &ctx)
}

fn scan_dir(path: &Path, name: Box<str>, ctx: &Ctx) -> Node {
    let mut node = Node { name, size: 0, files: 0, dir: true, children: vec![] };
    if ctx.p.cancel.load(Relaxed) {
        return node;
    }
    let Ok(rd) = fs::read_dir(path) else {
        ctx.p.denied.fetch_add(1, Relaxed);
        return node;
    };
    if let Ok(mut c) = ctx.p.current.try_lock() {
        *c = path.to_string_lossy().into_owned();
    }
    let entries: Vec<_> = rd.flatten().collect();
    node.children = entries
        .par_iter()
        .filter_map(|e| {
            let md = e.metadata().ok()?; // folgt Symlinks nicht
            let name: Box<str> = e.file_name().to_string_lossy().into();
            if md.is_dir() {
                let p = e.path();
                if !ctx.devs.contains(&md.dev()) || ctx.skip.contains(&p) {
                    return None; // andere Volumes (USB, Netzlaufwerke) nicht mitzählen
                }
                return Some(scan_dir(&p, name, ctx));
            }
            // Belegter Platz wie bei `du`, nicht die logische Länge (sparse Dateien, Klone).
            let mut size = md.blocks() * 512;
            if md.nlink() > 1 && !md.is_symlink() && !ctx.hardlinks.lock().unwrap().insert((md.dev(), md.ino())) {
                size = 0; // Hardlink schon gezählt
            }
            ctx.p.files.fetch_add(1, Relaxed);
            ctx.p.bytes.fetch_add(size, Relaxed);
            Some(Node { name, size, files: 1, dir: false, children: vec![] })
        })
        .collect();
    node.children.sort_unstable_by(|a, b| b.size.cmp(&a.size));
    node.children.shrink_to_fit();
    node.size = node.children.iter().map(|c| c.size).sum();
    node.files = node.children.iter().map(|c| c.files).sum();
    node
}

pub fn find<'a>(root: &'a Node, id: &[u32]) -> Option<&'a Node> {
    id.iter().try_fold(root, |n, &i| n.children.get(i as usize))
}

/// Die `n` größten Dateien als (Größe, Indexpfad).
pub fn largest_files(root: &Node, n: usize) -> Vec<(u64, Vec<u32>)> {
    fn walk(node: &Node, id: &mut Vec<u32>, out: &mut Vec<(u64, Vec<u32>)>, n: usize) {
        for (i, c) in node.children.iter().enumerate() {
            // Kinder sind absteigend sortiert: ist eins zu klein, sind es alle folgenden auch.
            if out.len() == n && c.size <= out[n - 1].0 {
                break;
            }
            id.push(i as u32);
            if c.dir {
                walk(c, id, out, n);
            } else {
                let pos = out.partition_point(|(s, _)| *s >= c.size);
                out.insert(pos, (c.size, id.clone()));
                out.truncate(n);
            }
            id.pop();
        }
    }
    let mut out = Vec::with_capacity(n + 1);
    walk(root, &mut vec![], &mut out, n);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scannt_sortiert_und_findet_groesste() {
        let d = std::env::temp_dir().join(format!("sb-test-{}", std::process::id()));
        fs::create_dir_all(d.join("a/b")).unwrap();
        fs::write(d.join("klein.txt"), vec![1u8; 10]).unwrap();
        fs::write(d.join("a/mittel.bin"), vec![1u8; 100_000]).unwrap();
        fs::write(d.join("a/b/gross.bin"), vec![1u8; 1_000_000]).unwrap();
        fs::hard_link(d.join("a/b/gross.bin"), d.join("link.bin")).unwrap();

        let root = scan(&d, &Progress::default());
        assert_eq!(root.files, 4);
        assert!(root.children.windows(2).all(|w| w[0].size >= w[1].size)); // größtes Kind zuerst
        assert!(root.size >= 1_100_000 && root.size < 1_300_000, "Hardlink doppelt gezählt: {}", root.size);
        let top = largest_files(&root, 2);
        assert_eq!(top.len(), 2);
        assert!(matches!(&*find(&root, &top[0].1).unwrap().name, "gross.bin" | "link.bin"));
        fs::remove_dir_all(&d).unwrap();
    }
}
