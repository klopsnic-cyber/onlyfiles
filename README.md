# Speicherblick

Zeigt wie TreeSize, welche Ordner und Dateien auf dem Mac den Platz belegen: Baumliste mit Balken, Ringdiagramm zum Hineinzoomen, die 100 größten Dateien.

- Scan parallel in Rust (rayon), zählt belegten Platz wie `du`, Hardlinks nur einmal, bleibt auf einem Volume.
- „Ganzer Mac“ (`/`) zählt die Daten-Seite über die Firmlinks und überspringt `/System/Volumes`, damit nichts doppelt zählt.
- Ordner ohne Leserecht werden gezählt und gemeldet; mit Festplattenvollzugriff fehlen keine mehr.

## Entwickeln
```
npm install
npm run tauri dev                  # SB_AUTOSCAN=$HOME startet direkt einen Scan (nur Dev)
cd src-tauri && cargo test
npm run tauri build                # .app und .dmg unter src-tauri/target/release/bundle
```
