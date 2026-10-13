<script>
  import { X } from "@lucide/svelte";
  import { Tween } from "svelte/motion";
  import { bytes, num } from "./api.js";

  let { target, progress, oncancel } = $props();

  // Zähler gleiten zwischen den 100-ms-Meldungen, statt zu springen.
  const b = new Tween(0, { duration: 250 });
  const f = new Tween(0, { duration: 250 });
  $effect(() => {
    if (!progress) return;
    b.target = progress.bytes;
    f.target = progress.files;
  });
  let rate = $derived(progress?.secs > 0.3 ? progress.files / progress.secs : 0);
</script>

<div class="scan">
  <div class="orbit">
    <i class="r1"></i><i class="r2"></i><i class="r3"></i>
    <span class="core"></span>
  </div>

  <div class="big num">{bytes(b.current)}</div>
  <div class="stats">
    <span><b class="num">{num(f.current)}</b> Dateien</span>
    <span><b class="num">{num(rate)}</b> / Sekunde</span>
    <span><b class="num">{(progress?.secs ?? 0).toFixed(1)}</b> s</span>
  </div>
  <div class="path">{progress?.current || target}</div>

  <button onclick={oncancel}><X size={15} /> Abbrechen</button>
</div>

<style>
  .scan { height: 100%; display: flex; flex-direction: column; align-items: center; justify-content: center; gap: 14px; padding-bottom: 6vh; animation: fade 0.5s var(--ease) both; }
  .orbit { position: relative; width: 210px; height: 210px; margin-bottom: 18px; }
  .orbit i { position: absolute; inset: 0; border-radius: 50%; border: 3px solid transparent; }
  .r1 { border-top-color: #00b4f5 !important; border-right-color: #00b4f555 !important; animation: spin 1.6s linear infinite; }
  .r2 { inset: 26px !important; border-top-color: #22c3ee !important; border-left-color: #22c3ee55 !important; animation: spin 2.3s linear infinite reverse; }
  .r3 { inset: 52px !important; border-bottom-color: #ff8f80 !important; border-right-color: #ff8f8055 !important; animation: spin 3.1s linear infinite; }
  .core { position: absolute; inset: 82px; border-radius: 50%; background: var(--grad); box-shadow: 0 0 0 0 rgba(0, 150, 240, 0.45); animation: pulse 1.8s ease-out infinite; }
  .big { font-size: 46px; font-weight: 700; letter-spacing: -0.03em; background: var(--grad); -webkit-background-clip: text; background-clip: text; color: transparent; }
  .stats { display: flex; gap: 26px; color: var(--muted); }
  .stats b { color: var(--text); font-weight: 600; }
  .path {
    max-width: 640px; width: 100%; text-align: center; font: 12px ui-monospace, Menlo, monospace; color: var(--muted);
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis; direction: rtl; margin-bottom: 10px;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  @keyframes pulse { 70% { box-shadow: 0 0 0 26px rgba(0, 150, 240, 0); } 100% { box-shadow: 0 0 0 0 rgba(0, 150, 240, 0); } }
  @keyframes fade { from { opacity: 0; transform: scale(0.97); } }
</style>
