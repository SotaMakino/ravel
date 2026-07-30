// Build the site with ravel.
//
// ravel renders the HTML shell, writes the post data, and copies the client
// module. Preact draws the pages in the browser from there.
//
//   ravel --build examples/site/build.tsx

import { Shell } from "./components.tsx";

const BASE = "/ravel/";

// Pinned, because an import map is a lockfile in disguise: nothing here is
// resolved at build time, so the browser gets whatever these URLs say.
//
// The leading `*` on htm marks its dependencies as external, so its internal
// `from "preact"` resolves through this map too. Without it esm.sh inlines its
// own preact URL and the page ends up with two copies of Preact, which quietly
// breaks hook state.
const IMPORT_MAP = {
  imports: {
    preact: "https://esm.sh/preact@10.29.7",
    "preact/hooks": "https://esm.sh/preact@10.29.7/hooks",
    htm: "https://esm.sh/htm@3.1.1",
    "htm/preact": "https://esm.sh/*htm@3.1.1/preact",
  },
};

const posts = [
  {
    slug: "hello-world",
    title: "Hello World",
    date: "2026-01-15",
    body: "First post on my new site.",
  },
  {
    slug: "ravel-intro",
    title: "Building with Ravel",
    date: "2026-02-01",
    body: "How I built this site using Ravel's SSG capabilities.",
  },
  {
    slug: "deployment",
    title: "Going Live",
    date: "2026-03-10",
    body: "Tips for deploying your static site.",
  },
];

const CSS = [
  "* { margin: 0; padding: 0; box-sizing: border-box; }",
  "body { font-family: system-ui, sans-serif; line-height: 1.6; color: #222; max-width: 800px; margin: 0 auto; padding: 1rem; }",
  "header { border-bottom: 2px solid #eee; padding: 1rem 0; margin-bottom: 2rem; }",
  "nav a { margin-right: 1rem; text-decoration: none; color: #0066cc; }",
  "nav a:hover { text-decoration: underline; }",
  "nav a[aria-current='page'] { font-weight: 600; color: #222; }",
  "main { min-height: 60vh; }",
  "footer { border-top: 2px solid #eee; padding: 1rem 0; margin-top: 2rem; color: #888; font-size: 0.9rem; }",
  ".post-card { margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #eee; }",
  ".post-card h2 { margin-bottom: 0.25rem; }",
  ".post-card a { color: #0066cc; text-decoration: none; }",
  ".post-card a:hover { text-decoration: underline; }",
  "time { color: #999; font-size: 0.85rem; }",
  "article { margin-bottom: 1.5rem; }",
  "h1 { margin-bottom: 0.5rem; }",
  ".filter { display: block; margin-bottom: 0.5rem; font-size: 0.9rem; color: #555; }",
  ".filter input { display: block; width: 100%; margin-top: 0.25rem; padding: 0.5rem; font: inherit; border: 1px solid #ccc; border-radius: 4px; }",
  ".count { color: #999; font-size: 0.85rem; margin-bottom: 1.5rem; }",
  ".error { color: #b00; }",
].join("\n");

if (!ravel.build) {
  console.error(
    "This script must be run in build mode: ravel --build build.tsx",
  );
} else {
  console.log("Building with ravel v" + ravel.version);

  fs.mkdirSync("dist");

  await fs.writeFile("dist/style.css", CSS);
  console.log("wrote dist/style.css");

  // The post list the browser fetches, so the data has one source of truth.
  await fs.writeFile("dist/posts.json", JSON.stringify(posts, null, 2));
  console.log("wrote dist/posts.json (" + posts.length + " posts)");

  // No bundling and no transform: app.js is already an ES module the browser
  // can run, so copying it is the whole build step.
  const app = await fs.readFile("app.js");
  await fs.writeFile("dist/app.js", app);
  console.log("wrote dist/app.js");

  // One page. Routing happens in the hash, because static hosting has no SPA
  // fallback and /about would 404 on reload.
  const html = (
    <Shell
      title="ravel"
      base={BASE}
      version={ravel.version}
      importMap={JSON.stringify(IMPORT_MAP)}
    />
  );
  await fs.writeFile("dist/index.html", "<!DOCTYPE html>" + html);
  console.log("wrote dist/index.html");

  console.log("done - 1 page, 1 CSS, 1 module, 1 data file");
}
