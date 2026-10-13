<script>
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { ShieldAlert, RefreshCw, FolderOpen, ChartPie, FileStack, ChevronRight } from "@lucide/svelte";
  import TreeNode from "./TreeNode.svelte";
  import Sunburst from "./Sunburst.svelte";
  import { api, bytes, num, pct, PALETTE } from "./api.js";

  let { target, done, onrescan } = $props();

  let root = $state(null);
  let focus = $state([]);
  let chart = $state(null);
  let crumbs = $state([]);
  let tab = $state("chart");
  let largest = $state(null);

  onMount(async () => {
    root = await api.tree([], 1);
  });

  $effect(() => {
    const id = focus;
    api.tree(id, 4, 40, 0.004).then((d) => (chart = d));
    api.fullPath(id).then((full) => {
      const rel = full.slice(target.length).split("/").filter(Boolean);
      crumbs = [[root?.name ?? target, []], ...rel.map((n, i) => [n, id.slice(0, i + 1)])];
    });
  });

  async function showLargest() {
    tab = "files";
    largest ??= await api.largestFiles(100);
  }

  const reveal = async (id) => revealItemInDir(await api.fullPath(id));
</script>

<div class="result" in:fade={{ duration: 300 }}>
  <section class="kpis">
    <div class="kpi card"><span>Belegt</span><b class="num">{bytes(done.bytes)}</b></div>
    <div class="kpi card"><span>Dateien</span><b class="num">{num(done.files)}</b></div>
    <div class="kpi card"><span>Dauer</span><b class="num">{done.secs.toLocaleString("de-DE", { maximumFractionDigits: 1 })} s</b></div>
    {#if done.volume}
      <div class="kpi card"><span>Frei auf dem Volume</span><b class="num free">{bytes(done.volume.free)}</b></div>
    {/if}
    <button class="ghost" onclick={onrescan} title="Erneut scannen"><RefreshCw size={15} /></button>
  </section>

  {#if done.denied > 0}
    <div class="warn card">
      <ShieldAlert size={18} />
      <span><b>{num(done.denied)} Ordner</b> konnten nicht gelesen werden; ihr Inhalt fehlt in der Summe. Mit „Festplattenvollzugriff“ für Speicherblick wird alles gezählt.</span>
      <button onclick={() => api.openPrivacySettings()}>Einstellungen öffnen</button>
    </div>
  {/if}

  <section class="main">
    <div class="card tree">
      <div class="thead">
        <span>Name</span><span></span><span class="r">Anteil</span><span class="r">Größe</span><span class="r">Dateien</span>
      </div>
      <div class="scroll" role="tree">
        {#if root}
          <TreeNode node={root} parentSize={root.size} {focus} onfocus={(id) => (focus = id)} />
        {/if}
      </div>
      <p class="hint muted">Klick zeigt den Ordner im Diagramm · Doppelklick öffnet ihn im Finder</p>
    </div>

    <div class="card side">
      <div class="tabs">
        <button class="ghost" class:on={tab === "chart"} onclick={() => (tab = "chart")}><ChartPie size={15} /> Diagramm</button>
        <button class="ghost" class:on={tab === "files"} onclick={showLargest}><FileStack size={15} /> Größte Dateien</button>
      </div>

      {#if tab === "chart"}
        <nav class="crumbs">
          {#each crumbs as [name, id], i}
            {#if i}<ChevronRight size={13} class="muted" />{/if}
            <button class="ghost" class:last={i === crumbs.length - 1} onclick={() => (focus = id)}>{name}</button>
          {/each}
          {#if focus.length}<button class="ghost open" onclick={() => reveal(focus)} title="Im Finder zeigen"><FolderOpen size={14} /></button>{/if}
        </nav>
        <div class="chart">
          {#if chart}
            <Sunburst data={chart} onzoom={(n) => (focus = n.id)} onup={() => (focus = focus.slice(0, -1))} />
          {/if}
        </div>
      {:else}
        <ol class="largest">
          {#each largest ?? [] as f, i (f.id.join("/"))}
            <li style="--d:{Math.min(i, 20) * 25}ms; --c:{PALETTE[i % PALETTE.length]}" ondblclick={() => reveal(f.id)}>
              <span class="rank num">{i + 1}</span>
              <span class="fn"><b title={f.name}>{f.name}</b><small class="muted" title={f.dir}>{f.dir}</small></span>
              <span class="fbar"><i style:width="{pct(f.size, largest[0].size)}%"></i></span>
              <span class="num fs">{bytes(f.size)}</span>
              <button class="ghost" onclick={() => reveal(f.id)} title="Im Finder zeigen"><FolderOpen size={14} /></button>
            </li>
          {/each}
        </ol>
      {/if}
    </div>
  </section>
</div>

<style>
  .result { height: 100%; display: flex; flex-direction: column; gap: 14px; }
  .kpis { display: flex; gap: 12px; align-items: stretch; }
  .kpi { padding: 12px 18px; display: flex; flex-direction: column; gap: 2px; min-width: 150px; animation: rise 0.5s var(--ease) backwards; }
  .kpi:nth-child(2) { animation-delay: 60ms; } .kpi:nth-child(3) { animation-delay: 120ms; } .kpi:nth-child(4) { animation-delay: 180ms; }
  .kpi span { font-size: 11px; text-transform: uppercase; letter-spacing: 0.08em; color: var(--muted); font-weight: 600; }
  .kpi b { font-size: 22px; font-weight: 700; letter-spacing: -0.02em; }
  .kpi .free { color: #16a37a; }
  .kpis > button { margin-left: auto; align-self: center; }
  .warn { display: flex; gap: 12px; align-items: center; padding: 10px 14px; color: #8a5a00; background: #fff8e6; border-color: #f6dfa4; font-size: 13px; animation: rise 0.5s var(--ease) backwards; }
  .warn span { flex: 1; }

  .main { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1.25fr) minmax(360px, 1fr); gap: 14px; }
  .tree, .side { display: flex; flex-direction: column; min-height: 0; padding: 10px; }
  .thead {
    display: grid; grid-template-columns: 48px minmax(80px, 1fr) minmax(70px, 22%) 54px 74px 70px; gap: 6px;
    padding: 4px 10px 8px; font-size: 11px; text-transform: uppercase; letter-spacing: 0.07em; color: var(--muted); font-weight: 600;
    border-bottom: 1px solid var(--border); margin-bottom: 4px;
  }
  .thead span:first-child { grid-column: 1 / 3; padding-left: 14px; }
  .r { text-align: right; }
  .scroll { flex: 1; overflow: auto; }
  .hint { font-size: 11.5px; margin: 6px 6px 0; }

  .tabs { display: flex; gap: 4px; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
  .tabs button { color: var(--muted); padding: 6px 12px; }
  .tabs button.on { color: var(--accent); background: color-mix(in srgb, var(--accent) 10%, white); }
  .crumbs { display: flex; align-items: center; flex-wrap: wrap; gap: 0; padding: 8px 2px 0; font-size: 12.5px; }
  .crumbs button { padding: 3px 7px; font-weight: 500; color: var(--muted); border-radius: 7px; }
  .crumbs button.last { color: var(--text); font-weight: 650; }
  .crumbs .open { margin-left: auto; }
  .chart { flex: 1; min-height: 0; display: grid; place-items: center; padding: 14px; }

  .largest { list-style: none; margin: 6px 0 0; padding: 0; overflow: auto; flex: 1; }
  .largest li {
    display: grid; grid-template-columns: 26px minmax(0, 1fr) 70px 70px 30px; gap: 10px; align-items: center;
    padding: 7px 8px; border-radius: 10px; animation: rise 0.45s var(--ease) backwards; animation-delay: var(--d);
  }
  .largest li:hover { background: var(--surface-2); }
  .rank { color: var(--muted); font-size: 12px; text-align: right; }
  .fn { display: flex; flex-direction: column; min-width: 0; }
  .fn b, .fn small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .fn b { font-weight: 550; }
  .fn small { font-size: 11px; direction: rtl; text-align: left; }
  .fbar { height: 6px; background: var(--surface-2); border-radius: 4px; overflow: hidden; }
  .fbar i { display: block; height: 100%; background: var(--c); border-radius: 4px; animation: bar 0.9s var(--ease) backwards; animation-delay: var(--d); }
  .fs { text-align: right; font-weight: 600; }
  .largest button { padding: 4px; }
  @keyframes rise { from { opacity: 0; transform: translateY(10px); } }
  @keyframes bar { from { width: 0 !important; } }
</style>
