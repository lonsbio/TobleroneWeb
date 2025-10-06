import Aioli from "@biowasm/aioli";

async function boot() {
  // Optional: guard in case COEP/COOP isn't ready on first load
  if (!self.crossOriginIsolated) {
    console.warn("Page is not crossOriginIsolated; threads may be unavailable.");
  }



const a = await new Aioli([{tool: "tinyt",
   // version: "0.0.6",
   // program: "fastqe",         // Optional: sub-tool name; not needed for most tools (default: same as tool name)
    urlPrefix: "https://lonsbio.github.io/TobleroneWeb/",  // Optional: custom path to .wasm assets (default: biowasm CDN)
 //   loading: "eager",         // Optional: if set to "lazy", only downloads WebAssembly modules when needed, not at initialization (default: eager)
 //   reinit: false,           // Optional: if set to true, will reinitialize module after each invocation; not needed for most tools
}]);

  // e.g., list tools or run a simple command if configured in your setup
  // const res = await a.exec("echo Hello from Aioli");
  // document.getElementById("app").textContent = res.stdout || "Ready";
  document.getElementById("app").textContent = "Aioli ready";
}

boot().catch(err => {
  console.error(err);
  document.getElementById("app").textContent = "Error: " + err;
});