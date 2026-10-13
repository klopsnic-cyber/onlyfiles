<script>
  import { Tween } from "svelte/motion";
  import { cubicOut } from "svelte/easing";
  import { PALETTE, bytes, num } from "./api.js";

  // Ringdiagramm: Mitte = aktueller Ordner, jeder Ring eine Ebene tiefer.
  let { data, onzoom, onup, onhover } = $props();

  const R0 = 78, R1 = 250, TAU = Math.PI * 2;
  let depth = $derived(Math.max(1, maxDepth(data)));
  let ring = $derived((R1 - R0) / depth);
  let arcs = $derived(data ? layout(data) : []);
  let hover = $state(null);
  let shown = $derived(hover ?? data);
  let total = new Tween(0, { duration: 600, easing: cubicOut });
  $effect(() => { total.target = shown?.size ?? 0; });

  function maxDepth(n, d = 0) {
    return n?.children?.length ? Math.max(...n.children.map((c) => maxDepth(c, d + 1))) : d;
  }

  function layout(root) {
    const out = [];
    (function walk(n, a0, a1, d, color) {
      let a = a0;
      n.children?.forEach((c, i) => {
        const span = n.size ? ((a1 - a0) * c.size) / n.size : 0;
        const col = d === 0 ? PALETTE[i % PALETTE.length] : color;
        if (span > 0.003) {
          out.push({ n: c, a0: a, a1: a + span, d, col, i });
          if (c.children) walk(c, a, a + span, d + 1, col);
        }
        a += span;
      });
    })(root, 0, TAU, 0, null);
    return out;
  }

  function path(a0, a1, r0, r1) {
    a1 = Math.min(a1, a0 + TAU - 1e-4);
    const pad = Math.min(0.006, (a1 - a0) / 4);
    a0 += pad; a1 -= pad;
    const p = (a, r) => `${(Math.sin(a) * r).toFixed(2)},${(-Math.cos(a) * r).toFixed(2)}`;
    const big = a1 - a0 > Math.PI ? 1 : 0;
    return `M${p(a0, r1)}A${r1},${r1} 0 ${big} 1 ${p(a1, r1)}L${p(a1, r0)}A${r0},${r0} 0 ${big} 0 ${p(a0, r0)}Z`;
  }

  const fill = (a) => (a.n.rest ? "#dfe2ec" : `color-mix(in srgb, ${a.col} ${100 - a.d * 17}%, white)`);
  const key = (n) => n?.id?.join("/") ?? "";

  function enter(a) { hover = a.n; onhover?.(a.n); }
  function leave() { hover = null; onhover?.(null); }
</script>

<div class="wrap">
  <svg viewBox="-260 -260 520 520">
    {#key key(data)}
      <g class="rings">
        {#each arcs as a (key(a.n) || `r${a.d}-${a.a0}`)}
          <path
            d={path(a.a0, a.a1, R0 + a.d * ring + 1.5, R0 + (a.d + 1) * ring)}
            style:fill={fill(a)}
            style:animation-delay="{a.d * 90 + Math.min(a.i, 12) * 18}ms"
            class:dim={hover && hover !== a.n}
            class:click={a.n.dir && !a.n.rest}
            role="presentation"
            onmouseenter={() => enter(a)}
            onmouseleave={leave}
            onclick={() => a.n.dir && !a.n.rest && onzoom(a.n)}
          />
        {/each}
      </g>
    {/key}
    <circle r={R0 - 6} class="hub" class:up={data?.id?.length} role="presentation" onclick={() => data?.id?.length && onup()} />
  </svg>
  <div class="center">
    <small>{shown?.name}</small>
    <b class="num">{bytes(total.current)}</b>
    <span class="muted num">{num(shown?.files ?? 0)} Dateien</span>
    {#if !hover && data?.id?.length}<em>↑ eine Ebene hoch</em>{/if}
  </div>
</div>

<style>
  .wrap { position: relative; width: 100%; aspect-ratio: 1; max-height: 100%; margin: 0 auto; }
  svg { width: 100%; height: 100%; overflow: visible; }
  path {
    stroke: white; stroke-width: 1; transform-box: view-box; transform-origin: center;
    transition: opacity 0.25s, transform 0.3s var(--ease), filter 0.3s;
    animation: grow 0.75s var(--ease) backwards;
  }
  path.click { cursor: pointer; }
  path:hover { transform: scale(1.025); filter: drop-shadow(0 4px 10px rgba(30, 35, 70, 0.22)); }
  path.dim { opacity: 0.45; }
  .hub { fill: white; filter: drop-shadow(0 4px 14px rgba(30, 35, 70, 0.1)); transition: transform 0.3s var(--ease); transform-box: view-box; transform-origin: center; }
  .hub.up { cursor: pointer; }
  .hub.up:hover { transform: scale(1.05); }
  .center {
    position: absolute; inset: 0; display: flex; flex-direction: column; align-items: center; justify-content: center;
    pointer-events: none; text-align: center; gap: 1px;
  }
  .center small { max-width: 120px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; color: var(--muted); font-size: 12px; }
  .center b { font-size: 21px; font-weight: 700; letter-spacing: -0.02em; }
  .center span { font-size: 11.5px; }
  .center em { font-style: normal; font-size: 10.5px; color: var(--accent); margin-top: 2px; }
  @keyframes grow { from { opacity: 0; transform: scale(0.82) rotate(-25deg); } }
</style>
