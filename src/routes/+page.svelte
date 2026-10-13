<script>
  import "$lib/theme.css";
  import { onMount } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { RotateCcw, Download, X } from "@lucide/svelte";
  import { check } from "@tauri-apps/plugin-updater";
  import { relaunch } from "@tauri-apps/plugin-process";
  import { getVersion } from "@tauri-apps/api/app";
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
  let version = $state("");
  let update = $state(null);
  let updating = $state(false);
  let updateErr = $state("");

  onMount(() => {
    const log = (m) => invoke("frontend_log", { msg: String(m) }).catch(() => {});
    window.addEventListener("error", (e) => log(`${e.message} @ ${e.filename}:${e.lineno}`));
    window.addEventListener("unhandledrejection", (e) => log(`Promise: ${e.reason?.stack || e.reason}`));
    api.places().then((p) => (places = p));
    getVersion().then((v) => (version = v));
    checkUpdate();
    const iv = setInterval(() => !update && checkUpdate(), 4 * 3600 * 1000); // App bleibt oft lange offen
    invoke("dev_autoscan").then((p) => p && scan(p));
    const un = [
      listen("scan-progress", (e) => (progress = e.payload)),
      listen("scan-done", (e) => {
        done = e.payload;
        view = e.payload.cancelled ? "start" : "result";
      }),
    ];
    return () => {
      clearInterval(iv);
      un.forEach((p) => p.then((f) => f()));
    };
  });

  async function checkUpdate() {
    try {
      update = await check();
    } catch {} // offline oder noch kein Release: still weiter
  }

  async function installUpdate() {
    updating = true;
    updateErr = "";
    try {
      await update.downloadAndInstall();
      await relaunch();
    } catch (e) {
      updateErr = String(e);
      updating = false;
    }
  }

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
      <img class="logo" src="/logo.png" alt="" />
      <b data-tauri-drag-region>OF OnlyFiles</b>
      {#if version}<small class="ver">{version}</small>{/if}
    </div>
    {#if view === "result"}
      <button class="ghost" onclick={() => (view = "start")}><RotateCcw size={15} /> Neuer Scan</button>
    {/if}
  </header>

  {#if update}
    <div class="update">
      <Download size={15} /> Version {update.version} ist da.
      <button class="primary small" disabled={updating} onclick={installUpdate}>{updating ? "Wird installiert …" : "Jetzt aktualisieren"}</button>
      <button class="ghost small" onclick={() => (update = null)} aria-label="Später"><X size={14} /></button>
      {#if updateErr}<span class="err">{updateErr}</span>{/if}
    </div>
  {/if}

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
  .logo { width: 24px; height: 24px; border-radius: 6px; }
  .ver { color: var(--muted); font-size: 11.5px; font-weight: 500; }
  .update {
    margin: 0 24px 12px; padding: 8px 12px; display: flex; align-items: center; gap: 10px; justify-content: center; font-size: 13px;
    border-radius: 12px; background: #e3f5fe; border: 1px solid #b6e4fb; color: #055a8c; animation: drop 0.5s var(--ease) both;
  }
  .small { padding: 4px 11px; font-size: 12.5px; }
  .err { color: #c0392b; }
  @keyframes drop { from { opacity: 0; transform: translateY(-8px); } }
  main { flex: 1; min-height: 0; padding: 0 24px 22px; }

  .blobs { position: fixed; inset: 0; z-index: -1; overflow: hidden; filter: blur(70px); opacity: 0.55; }
  .blobs i { position: absolute; width: 460px; height: 460px; border-radius: 50%; animation: drift 26s ease-in-out infinite alternate; }
  .blobs i:nth-child(1) { background: #cdeeff; top: -160px; left: -120px; }
  .blobs i:nth-child(2) { background: #cfe6ff; bottom: -200px; right: -100px; animation-delay: -9s; }
  .blobs i:nth-child(3) { background: #d3f7ef; top: 40%; left: 45%; width: 340px; height: 340px; animation-delay: -17s; }
  @keyframes drift { to { transform: translate(90px, 60px) scale(1.15); } }
  @media (prefers-reduced-motion: reduce) { .blobs i { animation: none; } }
</style>
