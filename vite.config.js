import { defineConfig } from "vite";

// If deploying to https://<user>.github.io/<REPO>/ set base = "/<REPO>/"
// If using a custom domain at the root, set base = "/"
const base = process.env.GITHUB_REPOSITORY?.split("/")[1]
  ? `/${process.env.GITHUB_REPOSITORY.split("/")[1]}/`
  : "/TobleroneWeb";

export default defineConfig({
  base,
  build: { outDir: "pages" },
});
