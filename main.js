// --- Minimal main.js (clean baseline) ---

import Aioli from "@biowasm/aioli";

// Build an absolute base that matches your GH Pages repo path.
// Make sure vite.config.js has: base: "/TobleroneWeb/"
const ABS_BASE = new URL(import.meta.env.BASE_URL, location.origin).href; // e.g. https://lonsbio.github.io/TobleroneWeb/

// OPTIONAL: light fetch logger (safe)
const __origFetch = self.fetch.bind(self);
self.fetch = (...args) => {
  try { console.log("[fetch]", args[0]); } catch {}
  return __origFetch(...args);
};

document.addEventListener("DOMContentLoaded", () => {
  (async () => {
    // Confirm COI for threads
    if (!self.crossOriginIsolated) {
      console.warn("Not crossOriginIsolated; threads may be unavailable.");
    }

    const outEl = document.getElementById("output");
    const helpEl = document.getElementById("output_help");
    if (!outEl || !helpEl) {
      console.error("Missing #output or #output_help in HTML");
      return;
    }

    try {
      // One instance is enough; you can keep two if you really need to.
      const CLI = await new Aioli(
        [{ tool: "tinyt", urlPrefix: ABS_BASE }],
        { debug: true }
      );

      // Mount sample data
      await CLI.mount({
        name: "test.fq",
        data:
"@HWI-D00360:5:H814YADXX:1:2209:15175:39729 1:N:0:CGATGT\nTTGGAGGATTTTGGAGAATCCCCTTAGGGGGAAATGTTTAAAAGTGCAAAGTGAATAGTAGAAGCCCCTCTCCTCGTCACTAGGGGTACATTTGCCGTTTTCTTATCAACAGCCTCTCAAGTACAAGCATCTGGGACAAGAACTAGAA\n+\n@CCFFFFFHHHHHJIIIJJJJJJJJJJJJJJGIJIIIIJJJJIJCHIIJJJ@GGJJJIGIJJHGGHHFFFDEEEEDDDDDDDDDDD;@CDEEEEDDDBDDDDCDDDDECCDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDC\n"
      });

      const fasta_text =
        `>ENST00000331340.8|IKZF1|3|4|5|6|7|8|9
ACTCTAACAAGTGACTGCGCGGCCCGCGCCCGGGGCGGTGACTGCGGCAAGCCCCCTGGGTCCCCGCGC
`;
      await CLI.mount({ name: "test.fasta", data: fasta_text });

      // Run commands
      const output_help = await CLI.exec("tinyt --help");
      // adjust the path/index name to your actual asset names on Pages:
      const output = await CLI.exec("tinyt index --num-threads=1 -i /toblerone/testindex.idx test.fasta");

      // Update DOM
      helpEl.textContent = output_help;
      outEl.textContent = output;

    } catch (err) {
      console.error(err);
      const app = document.getElementById("app") || document.body;
      app.textContent = "Error: " + (err && err.message ? err.message : err);
    }
  })();
});