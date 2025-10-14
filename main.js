import Aioli from "@biowasm/aioli";

const ABS_BASE = new URL(import.meta.env.BASE_URL, location.origin).href;

document.addEventListener("DOMContentLoaded", async () => {
  const out  = document.getElementById("output");
  const help = document.getElementById("output_help");
  const show = (el, label, r) => {
    const o = typeof r === "string" ? { stdout: r } : (r || {});
    el.textContent += `\n# ${label}\nSTDOUT:\n${o.stdout ?? ""}\nSTDERR:\n${o.stderr ?? ""}\n(code=${o.code ?? "?"})\n`;
  };
console.log("start try");

// 1) See any silent page-level errors
window.addEventListener('error',  e => console.error('[page error]', e.message, e.error || ''));
window.addEventListener('unhandledrejection', e => console.error('[unhandled]', e.reason));

// 2) Log every Worker creation + errors (doesn’t change behavior)
(() => {
  const OrigWorker = window.Worker;
  window.Worker = function(url, opts) {
    const w = new OrigWorker(url, opts);
    console.log('[Worker new]', url);
    w.addEventListener('error',        ev => console.error('[Worker error]', url, ev.message || ev));
    w.addEventListener('messageerror', ev => console.error('[Worker messageerror]', url, ev));
    return w;
  };
})();

// 3) Timeout guard so we can see a “hung” message if Aioli doesn’t resolve
const withTimeout = (p, ms, label='op') => Promise.race([
  p,
  new Promise((_, rej) => setTimeout(() => rej(new Error(`[timeout] ${label} > ${ms}ms`)), ms))
]);

// 4) Ensure the SW has taken control (threads need crossOriginIsolated === true)
async function ensureIsolation(scopeBase) {
  if (location.protocol === 'https:' && 'serviceWorker' in navigator && !crossOriginIsolated) {
    // Wait for activation/claim
    await navigator.serviceWorker.ready.catch(()=>{});
    for (let i=0;i<80 && !crossOriginIsolated;i++) { // up to ~8s
      await new Promise(r=>setTimeout(r,100));
    }
  }
  console.log('[isolation]', { crossOriginIsolated });
}

  try {
    console.log("in try");

    const CLI = await new Aioli(
      [{
         tool: "tinyt",

         // Optional: sub-tool name; not needed for most tools (default: same as tool name)
  urlPrefix: 'https://lonsbio.github.io/TobleroneWeb/',  // Optional: custom path to .wasm assets (default: biowasm CDN)
    loading: "eager",         // Optional: if set to "lazy", only downloads WebAssembly modules when needed, not at initialization (default: eager)
    reinit: false,           // Optional: if set to true, will reinitialize module after each invocation; not needed for most tools
}], {
    printInterleaved: true,  // Optional: whether to return interleaved stdout/stderr; if false, returns object with stdout/stderr keys (default: true)
    debug: true,            // Optional: set to true to see console log messages for debugging (default: false)
});
    
console.log("afetr CLI");
console.log(CLI);

    // mount inputs
    await CLI.mount({
      name: "test.fasta",
      data: `>ENST00000331340.8|IKZF1|3|4|5|6|7|8|9
ACTCTAACAAGTGACTGCGCGGCCCGCGCCCGGGGCGGTGACTGCGGCAAGCCCCCTGGGTCCCCGCGC
`
    });

    // if your index lives in the repo, mount by URL (adjust the path!)
    await CLI.mount({
      name: "testindex.idx",
      url:  ABS_BASE + "toblerone/testindex.idx"
    });

    // run tinyt; these WILL produce output
    show(help, "tinyt --version", await CLI.exec("tinyt --version"));
    show(help, "tinyt --help",    await CLI.exec("tinyt --help"));

    // run your command (note: -i argument must match the mounted filename)
    const res = await CLI.exec("tinyt index --num-threads=1 -i testindex.idx test.fasta");
    show(out, "tinyt index", res);

  } catch (e) {
    (out || document.body).textContent = "Error: " + (e?.message || e);
    console.error(e);
  }
});