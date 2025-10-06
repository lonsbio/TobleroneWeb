import Aioli from "@biowasm/aioli";

async function boot() {
  // Optional: guard in case COEP/COOP isn't ready on first load
  if (!self.crossOriginIsolated) {
    console.warn("Page is not crossOriginIsolated; threads may be unavailable.");
  }

  const a = await Aioli();
  // e.g., list tools or run a simple command if configured in your setup
  // const res = await a.exec("echo Hello from Aioli");
  // document.getElementById("app").textContent = res.stdout || "Ready";
  document.getElementById("app").textContent = "Aioli ready";
}

boot().catch(err => {
  console.error(err);
  document.getElementById("app").textContent = "Error: " + err;
});