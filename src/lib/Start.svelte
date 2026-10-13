<script>
  import { open } from "@tauri-apps/plugin-dialog";
  import { House, HardDrive, Download, AppWindow, FolderSearch, ChevronRight } from "@lucide/svelte";
  import { Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { bytes, pctText } from "./api.js";

  let { places, onscan } = $props();

  let used = $derived(places?.volume ? places.volume.total - places.volume.free : 0);
  let share = new Tween(0, { duration: 1400, easing: cubicOut });
  $effect(() => {
    if (places?.volume) share.target = used / places.volume.total;
  });

  let targets = $derived(
    places
      ? [
          [House, "Benutzerordner", places.home, "Dokumente, Downloads, Mediathek …", "#7c5cff"],
          [HardDrive, "Ganzer Mac", "/", "Alles auf Macintosh HD", "#4f8cff"],
          [Download, "Downloads", `${places.home}/Downloads`, "Oft der schnellste Gewinn", "#22b8d6"],
          [AppWindow, "Programme", "/Applications", "Installierte Apps", "#ff8a6b"],
        ]
      : [],
  );

  async function pick() {
    const p = await open({ directory: true, multiple: false, title: "Ordner zum Analysieren wählen" });
    if (p) onscan(p);
  }

  const R = 92, C = 2 * Math.PI * R;
</script>

<div class="start">
  <section class="hero">
    <div class="ring">
      <svg viewBox="0 0 220 220">
        <defs>
          <linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#00b4f5" /><stop offset="1" stop-color="#0077d9" /></linearGradient>
        </defs>
        <circle cx="110" cy="110" r={R} class="track" />
        <circle cx="110" cy="110" r={R} class="bar" stroke-dasharray="{C * share.current} {C}" />
      </svg>
      <div class="center">
        <b class="num">{pctText(share.current * 100)}</b>
        <span class="muted">belegt</span>
      </div>
    </div>
    <div class="intro">
      <h1>Wo ist mein Speicherplatz hin?</h1>
      {#if places?.volume}
        <p class="muted">
          Macintosh HD: <b class="num">{bytes(used)}</b> von {bytes(places.volume.total)} belegt,
          <b class="num free">{bytes(places.volume.free)}</b> frei.
        </p>
      {/if}
      <p class="muted">Wähle, was untersucht werden soll. Der Scan läuft auf allen Kernen und dauert meist nur Sekunden.</p>
    </div>
  </section>

  <section class="grid">
    {#each targets as [Icon, title, path, hint, color], i}
      <button class="target card" style="--c:{color}; --d:{i * 70}ms" onclick={() => onscan(path)}>
        <span class="ic"><Icon size={22} /></span>
        <span class="txt"><b>{title}</b><small class="muted">{hint}</small></span>
        <ChevronRight size={18} class="go" />
      </button>
    {/each}
    <button class="target card pick" style="--c:#8a90a8; --d:{targets.length * 70}ms" onclick={pick}>
      <span class="ic"><FolderSearch size={22} /></span>
      <span class="txt"><b>Ordner wählen …</b><small class="muted">Auch externe Laufwerke und Netzlaufwerke</small></span>
      <ChevronRight size={18} class="go" />
    </button>
  </section>
</div>

<style>
  .start { max-width: 920px; margin: 0 auto; padding-top: 4vh; display: flex; flex-direction: column; gap: 34px; }
  .hero { display: flex; align-items: center; gap: 40px; animation: rise 0.7s var(--ease) both; }
  .ring { position: relative; width: 210px; height: 210px; flex-shrink: 0; }
  .ring svg { width: 100%; height: 100%; transform: rotate(-90deg); }
  .track { fill: none; stroke: #e8eaf3; stroke-width: 20; }
  .bar { fill: none; stroke: url(#g); stroke-width: 20; stroke-linecap: round; filter: drop-shadow(0 6px 10px rgba(0, 140, 230, 0.3)); }
  .center { position: absolute; inset: 0; display: grid; place-content: center; text-align: center; }
  .center b { font-size: 34px; font-weight: 700; letter-spacing: -0.02em; }
  .intro h1 { font-size: 30px; margin-bottom: 10px; letter-spacing: -0.025em; }
  .intro p { margin: 6px 0; line-height: 1.5; font-size: 15px; }
  .intro b { color: var(--text); }
  .free { color: #16a37a !important; }

  .grid { display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }
  .target {
    display: flex; align-items: center; gap: 14px; padding: 16px 18px; text-align: left; border-radius: 16px;
    animation: rise 0.6s var(--ease) backwards; animation-delay: calc(150ms + var(--d));
  }
  .target:hover { transform: translateY(-3px); box-shadow: var(--shadow-lg); border-color: color-mix(in srgb, var(--c) 35%, var(--border)); }
  .ic { width: 46px; height: 46px; border-radius: 13px; display: grid; place-items: center; color: var(--c); background: color-mix(in srgb, var(--c) 12%, white); transition: transform 0.35s var(--ease); }
  .target:hover .ic { transform: scale(1.08) rotate(-4deg); }
  .txt { flex: 1; display: flex; flex-direction: column; gap: 3px; }
  .txt b { font-size: 15px; }
  .txt small { font-weight: 450; }
  .target :global(.go) { color: var(--muted); transition: transform 0.3s var(--ease); }
  .target:hover :global(.go) { transform: translateX(4px); color: var(--c); }
  .pick { grid-column: 1 / -1; border-style: dashed; }
  @keyframes rise { from { opacity: 0; transform: translateY(14px); } }
</style>
