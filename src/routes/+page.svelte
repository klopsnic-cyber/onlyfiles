<script>
  import "$lib/theme.css";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { RotateCcw } from "@lucide/svelte";
  import Start from "$lib/Start.svelte";
  import Scanning from "$lib/Scanning.svelte";
  import Result from "$lib/Result.svelte";
  import { api } from "$lib/api.js";
  import { invoke } from "@tauri-apps/api/core";

  let view = $state("start"); // start | scan | result
  let places = $state(null);
  let target = $state("");
  let progress = $state(null);
  let done = $state(null);

  onMount(() => {
    const log = (m) => invoke("frontend_log", { msg: String(m) }).catch(() => {});
    window.addEventListener("error", (e) => log(`${e.message} @ ${e.filename}:${e.lineno}`));
    window.addEventListener("unhandledrejection", (e) => log(`Promise: ${e.reason?.stack || e.reason}`));
    api.places().then((p) => (places = p));
    invoke("dev_autoscan").then((p) => p && scan(p));
    const un = [
      listen("scan-progress", (e) => (progress = e.payload)),
      listen("scan-done", (e) => {
        done = e.payload;
        view = e.payload.cancelled ? "start" : "result";
      }),
    ];
    return () => un.forEach((p) => p.then((f) => f()));
  });

  function scan(path) {
    target = path;
    progress = null;
    done = null;
    view = "scan";
    api.startScan(path);
  }
</script>

<div class="blobs" aria-hidden="true"><i></i><i></i><i></i></div>

<div class="app">
  <header data-tauri-drag-region>
    <div class="brand" data-tauri-drag-region>
      <span class="logo"></span>
      <b data-tauri-drag-region>Speicherblick</b>
    </div>
    {#if view === "result"}
      <button class="ghost" onclick={() => (view = "start")}><RotateCcw size={15} /> Neuer Scan</button>
    {/if}
  </header>

  <main>
    {#if view === "start"}
      <Start {places} onscan={scan} />
    {:else if view === "scan"}
      <Scanning {target} {progress} oncancel={() => api.cancelScan()} />
    {:else if done}
      <Result {target} {done} onrescan={() => scan(target)} />
    {/if}
  </main>
</div>

<style>
  .app { position: relative; height: 100vh; display: flex; flex-direction: column; }
  header {
    height: 52px; flex-shrink: 0; display: flex; align-items: center; justify-content: space-between;
    padding: 0 16px 0 88px; /* Platz für die Ampel-Knöpfe */
  }
  .brand { display: flex; align-items: center; gap: 9px; font-size: 14.5px; }
  .logo { width: 20px; height: 20px; border-radius: 50%; background: conic-gradient(#7c5cff 0 62%, #22c3ee 0 88%, #ff8f80 0); mask: radial-gradient(circle, transparent 5px, #000 5.5px); }
  main { flex: 1; min-height: 0; padding: 0 24px 22px; }

  .blobs { position: fixed; inset: 0; z-index: -1; overflow: hidden; filter: blur(70px); opacity: 0.55; }
  .blobs i { position: absolute; width: 460px; height: 460px; border-radius: 50%; animation: drift 26s ease-in-out infinite alternate; }
  .blobs i:nth-child(1) { background: #d9d2ff; top: -160px; left: -120px; }
  .blobs i:nth-child(2) { background: #cfe6ff; bottom: -200px; right: -100px; animation-delay: -9s; }
  .blobs i:nth-child(3) { background: #d3f7ef; top: 40%; left: 45%; width: 340px; height: 340px; animation-delay: -17s; }
  @keyframes drift { to { transform: translate(90px, 60px) scale(1.15); } }
  @media (prefers-reduced-motion: reduce) { .blobs i { animation: none; } }
</style>
