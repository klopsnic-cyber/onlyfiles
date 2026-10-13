import { invoke } from "@tauri-apps/api/core";

export const api = {
  places: () => invoke("places"),
  startScan: (path) => invoke("start_scan", { path }),
  cancelScan: () => invoke("cancel_scan"),
  tree: (id, depth, limit = 300, minFrac = 0) => invoke("tree", { id, depth, limit, minFrac }),
  fullPath: (id) => invoke("full_path", { id }),
  largestFiles: (n) => invoke("largest_files", { n }),
  openPrivacySettings: () => invoke("open_privacy_settings"),
};

// Wie der Finder: Dezimal-Einheiten (1 GB = 1000³ Byte).
export function bytes(n) {
  const u = ["Byte", "KB", "MB", "GB", "TB"];
  let i = 0;
  while (n >= 1000 && i < u.length - 1) (n /= 1000), i++;
  return `${n.toLocaleString("de-DE", { maximumFractionDigits: n < 10 && i > 0 ? 1 : 0 })} ${u[i]}`;
}
export const num = (n) => Math.round(n).toLocaleString("de-DE");
export const pct = (a, b) => (b ? (a / b) * 100 : 0);
export const pctText = (p) => `${p.toLocaleString("de-DE", { maximumFractionDigits: p < 10 ? 1 : 0 })} %`;

export const PALETTE = ["#7c5cff", "#4f8cff", "#22c3ee", "#2dd4bf", "#34d399", "#a3d93a", "#fbbf24", "#ff9f6b", "#ff6b9a", "#d96bf0"];
