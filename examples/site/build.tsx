import { Layout } from "./components.tsx";
import { PostCard } from "./components.tsx";

function writePage(path: string, html: string) {
  fs.writeFileSync(path, "<!DOCTYPE html>" + html);
  console.log("wrote " + path);
}

var posts = [
  { slug: "hello-world", title: "Hello World", date: "2026-01-15", body: "First post on my new site." },
  { slug: "ravel-intro", title: "Building with Ravel", date: "2026-02-01", body: "How I built this site using Ravel's SSG capabilities." },
  { slug: "deployment", title: "Going Live", date: "2026-03-10", body: "Tips for deploying your static site." },
];

if (!ravel.build) {
  console.error("This script must be run in build mode: ravel --build build.tsx");
} else {
  console.log("Building with ravel v" + ravel.version);
  console.log("__filename: " + __filename);
  console.log("__dirname: " + __dirname);
  console.log("RAVEL_BUILD=" + process.env.RAVEL_BUILD);

  // Create dist directories
  fs.mkdirSync("dist/blog");

  // Generate CSS
  var css = [
    "* { margin: 0; padding: 0; box-sizing: border-box; }",
    "body { font-family: system-ui, sans-serif; line-height: 1.6; color: #222; max-width: 800px; margin: 0 auto; padding: 1rem; }",
    "header { border-bottom: 2px solid #eee; padding: 1rem 0; margin-bottom: 2rem; }",
    "nav a { margin-right: 1rem; text-decoration: none; color: #0066cc; }",
    "nav a:hover { text-decoration: underline; }",
    "main { min-height: 60vh; }",
    "footer { border-top: 2px solid #eee; padding: 1rem 0; margin-top: 2rem; color: #888; font-size: 0.9rem; }",
    ".post-card { margin-bottom: 2rem; padding-bottom: 1rem; border-bottom: 1px solid #eee; }",
    ".post-card h2 { margin-bottom: 0.25rem; }",
    ".post-card a { color: #0066cc; text-decoration: none; }",
    ".post-card a:hover { text-decoration: underline; }",
    "time { color: #999; font-size: 0.85rem; }",
    "article { margin-bottom: 1.5rem; }",
    "h1 { margin-bottom: 0.5rem; }",
  ].join("\n");
  fs.writeFileSync("dist/style.css", css);
  console.log("wrote dist/style.css");

  // Home page
  writePage("dist/index.html",
    <Layout title="Home">
      <h1>Welcome</h1>
      <p>A static site built with Ravel.</p>
      <h2>Recent Posts</h2>
      {posts.map(function(p) {
        return <PostCard slug={p.slug} title={p.title} date={p.date} excerpt={p.body} />;
      })}
    </Layout>
  );

  // About page
  writePage("dist/about.html",
    <Layout title="About">
      <h1>About</h1>
      <p>This site is generated using Ravel, a minimal JS/TSX runtime for static site generation.</p>
      <h2>Features</h2>
      <ul>
        <li>JSX templating with component composition</li>
        <li>TypeScript/TSX support</li>
        <li>ES module imports</li>
        <li>Sandboxed filesystem access</li>
      </ul>
      <p>Check out the <a href="blog">blog</a> for more.</p>
    </Layout>
  );

  // Blog index
  writePage("dist/blog/index.html",
    <Layout title="Blog">
      <h1>Blog</h1>
      {posts.map(function(p) {
        return <PostCard slug={p.slug} title={p.title} date={p.date} excerpt={p.body} />;
      })}
    </Layout>
  );

  // Individual blog posts
  for (var i = 0; i < posts.length; i++) {
    var post = posts[i];
    writePage("dist/blog/" + post.slug + ".html",
      <Layout title={post.title}>
        <h1>{post.title}</h1>
        <time>{post.date}</time>
        <p>{post.body}</p>
        <p><a href="./">Back to blog</a></p>
      </Layout>
    );
  }

  // Read back a generated file to verify
  var existing = fs.exists("dist/index.html");
  console.log("dist/index.html exists: " + existing);

  console.log("done - " + (3 + posts.length) + " pages, 1 CSS");
}