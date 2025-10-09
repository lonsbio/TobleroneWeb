import Aioli from "@biowasm/aioli";

const ABS_BASE = new URL(import.meta.env.BASE_URL, location.origin).href; // https://lonsbio.github.io/TobleroneWeb/

document.addEventListener("DOMContentLoaded", async () => {
  const out  = document.getElementById("output");
  const help = document.getElementById("output_help");
  const show = (el, label, r) => {
    const o = typeof r === "string" ? { stdout: r, stderr: "" } : (r || {});
    el.textContent += `\n# ${label}\nSTDOUT:\n${o.stdout || ""}\nSTDERR:\n${o.stderr || ""}\n(code=${o.code ?? "?"})\n`;
  };

  try {
    const CLI = await new Aioli(
      [{ tool: "tinyt", urlPrefix: ABS_BASE }],   // absolute, trailing slash
      { debug: true, returnType: "object" }       // capture stdout/stderr/code
    );

    // 1) Sanity: tool is callable
    show(help, "tinyt --version", await CLI.exec("tinyt --version"));

    // 2) Mount inputs explicitly
    await CLI.mount({ name: "test.fasta", data:
`>ENST00000331340.8|IKZF1|3|4|5|6|7|8|9
ACTCTAACAAGTGACTGCGCGGCCCGCGCCCGGGGCGGTGACTGCGGCAAGCCCCCTGGGTCCCCGCGC
`});

    // If your index file lives in the repo, mount it from an absolute URL:
    // (adjust the path/filename to where your real index file is in Pages)
    await CLI.mount({
      name: "testindex.idx",
      url:  ABS_BASE + "toblerone/testindex.idx"
    });

    // 3) Quick FS peek so we know the files are there
    const lsRoot = await CLI.exec("ls -la /");
    const lsTobl = await CLI.exec("ls -la /toblerone || true"); // may not exist
    const lsHome = await CLI.exec("ls -la .");
    show(help, "ls /", lsRoot);
    show(help, "ls /toblerone", lsTobl);
    show(help, "ls .", lsHome);

    // 4) Run help and your index command (paths now match what we mounted)
    show(help, "tinyt --help", await CLI.exec("tinyt --help"));
    const res = await CLI.exec("tinyt index --num-threads=1 -i testindex.idx test.fasta");
    show(out, "tinyt index", res);

  } catch (e) {
    out.textContent = "Error: " + (e?.message || e);
    console.error(e);
  }
});