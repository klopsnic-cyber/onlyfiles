<script>
  import TreeNode from "./TreeNode.svelte";
  import { onMount } from "svelte";
  import { slide } from "svelte/transition";
  import { ChevronRight, Folder, File, Ellipsis } from "@lucide/svelte";
  import { revealItemInDir } from "@tauri-apps/plugin-opener";
  import { api, bytes, num, pct, pctText, PALETTE } from "./api.js";

  // Eine Zeile wie bei TreeSize: Name, Balken (Anteil am Elternordner), Größe, Dateien.
  let { node, parentSize, level = 0, index = 0, color = null, focus, onfocus } = $props();

  // svelte-ignore state_referenced_locally (Startwert genügt, Kinder werden danach selbst geladen)
  let kids = $state(node.children ?? null);
  // svelte-ignore state_referenced_locally (nur die Wurzel startet aufgeklappt)
  let open = $state(level === 0);
  let width = $state(0);
  let p = $derived(pct(node.size, parentSize));
  let col = $derived(color ?? PALETTE[index % PALETTE.length]);
  let canOpen = $derived(node.dir && node.count > 0);
  let selected = $derived(node.id && focus?.join("/") === node.id.join("/"));

  onMount(() => requestAnimationFrame(() => (width = p)));
  $effect(() => { width = p; });

  async function toggle(e) {
    e?.stopPropagation();
    if (!canOpen) return;
    open = !open;
    if (open && !kids) kids = (await api.tree(node.id, 1))?.children ?? [];
  }

  async function reveal() {
    if (node.id) revealItemInDir(await api.fullPath(node.id));
  }
</script>

<div
  class="row"
  class:sel={selected}
  class:rest={node.rest}
  style="--lvl:{level}; --c:{node.rest ? '#b7bccd' : col}; --d:{Math.min(index, 14) * 22}ms"
  role="treeitem"
  aria-selected={selected}
  aria-expanded={canOpen ? open : undefined}
  tabindex="-1"
  onclick={() => node.dir && node.id && onfocus(node.id)}
  ondblclick={reveal}
  onkeydown={(e) => e.key === "Enter" && toggle()}
>
  <button class="chev ghost" class:open class:hidden={!canOpen} onclick={toggle} tabindex="-1" aria-label="Aufklappen">
    <ChevronRight size={14} />
  </button>
  <span class="ic">
    {#if node.rest}<Ellipsis size={15} />{:else if node.dir}<Folder size={15} />{:else}<File size={15} />{/if}
  </span>
  <span class="name" title={node.name}>{node.name}</span>
  <span class="bar"><i style:width="{width}%"></i></span>
  <span class="pct num">{pctText(p)}</span>
  <span class="size num">{bytes(node.size)}</span>
  <span class="files num muted">{node.dir || node.rest ? num(node.files) : ""}</span>
</div>

{#if open && kids}
  <div transition:slide={{ duration: 260 }}>
    {#each kids as k, i (k.id?.join("/") ?? "rest")}
      <TreeNode node={k} parentSize={node.size} level={level + 1} index={i} color={level === 0 ? null : col} {focus} {onfocus} />
    {/each}
  </div>
{/if}

<style>
  .row {
    display: grid; grid-template-columns: 22px 20px minmax(80px, 1fr) minmax(70px, 22%) 54px 74px 70px; align-items: center; gap: 6px;
    height: 34px; padding: 0 10px 0 calc(6px + var(--lvl) * 18px); border-radius: 9px; cursor: default;
    animation: in 0.45s var(--ease) backwards; animation-delay: var(--d);
    transition: background 0.15s;
  }
  .row:hover { background: var(--surface-2); }
  .row.sel { background: color-mix(in srgb, var(--c) 11%, white); box-shadow: inset 3px 0 0 var(--c); }
  .chev { padding: 2px; border-radius: 6px; color: var(--muted); transition: transform 0.25s var(--ease); }
  .chev.open { transform: rotate(90deg); }
  .chev.hidden { visibility: hidden; }
  .ic { color: var(--c); display: grid; place-items: center; }
  .name { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 500; }
  .rest .name { color: var(--muted); font-style: italic; font-weight: 450; }
  .bar { height: 8px; border-radius: 5px; background: var(--surface-2); overflow: hidden; }
  .bar i { display: block; height: 100%; border-radius: 5px; background: linear-gradient(90deg, color-mix(in srgb, var(--c) 70%, white), var(--c)); transition: width 0.9s var(--ease); }
  .pct { text-align: right; font-size: 12px; color: var(--muted); }
  .size { text-align: right; font-weight: 600; }
  .files { text-align: right; font-size: 12px; }
  @keyframes in { from { opacity: 0; transform: translateX(-8px); } }
</style>
