// download-biowasm-tools.mjs
// Run with:  node download-biowasm-tools.mjs

import fs from "fs";
import path from "path";
import { fileURLToPath } from "url";
import fetch from "node-fetch";

const __dirname = path.dirname(fileURLToPath(import.meta.url));

/** List of Biowasm tools to mirror locally */
const tools = [
  { tool: "coreutils", version: "8.32", program: "head" },
  // { tool: "tinyt", version: "0.0.6" },   // add more as needed
];

const CDN = "https://biowasm.com/cdn/v3/";

async function downloadFile(url, dest) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url} -> ${res.status}`);
  await fs.promises.mkdir(path.dirname(dest), { recursive: true });
  const buf = await res.arrayBuffer();
  await fs.promises.writeFile(dest, Buffer.from(buf));
  console.log("✔", dest);
}

for (const t of tools) {
  const base = `${t.tool}/${t.version}/${t.program || t.tool}`;
  const files = [`${base}.js`, `${base}.wasm`];
  for (const f of files) {
    const url = CDN + f;
    const dest = path.join(__dirname, "pages_temp", f); // match your Pages output dir
    try {
      await downloadFile(url, dest);
    } catch (e) {
      console.warn("✖", url, e.message);
    }
  }
}
