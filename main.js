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
  try {
    console.log("in try");

    const CLI = await new Aioli(
      [{ tool: "tinyt", urlPrefix: ABS_BASE }],
      { debug: true, returnType: "object" }
    );
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