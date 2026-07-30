// The client half of the site. Runs in the browser, not in ravel.
//
// No build step: preact and htm arrive as ES modules through the import map in
// index.html, and htm's tagged templates stand in for JSX. ravel's job is to
// emit the shell and the data this file reads.

import { render } from "preact";
import { useEffect, useState } from "preact/hooks";
import { html } from "htm/preact";

// Stamped onto <html> by the build script, so the footer reports the version of
// the binary that actually built the page.
const RAVEL_VERSION =
  document.documentElement.dataset.ravelVersion ?? "unknown";

// Routes live in the hash. GitHub Pages serves static files and has no SPA
// fallback, so /about would 404 on reload while #/about always works.
function useHashRoute() {
  const read = () => location.hash.replace(/^#\/?/, "") || "home";
  const [route, setRoute] = useState(read);
  useEffect(() => {
    const onChange = () => setRoute(read());
    addEventListener("hashchange", onChange);
    return () => removeEventListener("hashchange", onChange);
  }, []);
  return route;
}

// Written by the build script, so the post list has a single source of truth.
function usePosts() {
  const [state, setState] = useState({ status: "loading", posts: [] });
  useEffect(() => {
    fetch("./posts.json")
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.json();
      })
      .then((posts) => setState({ status: "ready", posts }))
      .catch((error) => setState({ status: "failed", error: error.message }));
  }, []);
  return state;
}

function Nav({ route }) {
  const links = [
    ["home", "Home"],
    ["about", "About"],
    ["blog", "Blog"],
  ];
  return html`
    <nav>
      ${links.map(
        ([target, label], i) => html`
          ${i > 0 ? " | " : ""}
          <a
            href=${`#/${target}`}
            aria-current=${route === target ? "page" : null}
            >${label}</a
          >
        `,
      )}
    </nav>
  `;
}

function PostCard({ post }) {
  return html`
    <article class="post-card">
      <h2><a href=${`#/blog/${post.slug}`}>${post.title}</a></h2>
      <time>${post.date}</time>
      <p>${post.body}</p>
    </article>
  `;
}

function Home({ posts }) {
  return html`
    <h1>Welcome</h1>
    <p>A site rendered in the browser by Preact, built by ravel.</p>
    <h2>Recent Posts</h2>
    ${posts.map((post) => html`<${PostCard} post=${post} />`)}
  `;
}

function About() {
  return html`
    <h1>About</h1>
    <p>
      ravel is a toy JavaScript runtime written in Rust. It built this page's
      shell, wrote the post data, and serves the result.
    </p>
    <h2>How this page works</h2>
    <ul>
      <li>ravel generated the HTML shell from TSX at build time</li>
      <li>Preact and htm load as ES modules through an import map</li>
      <li>Routing is hash-based, so reloads work on static hosting</li>
      <li>The post list is fetched from a JSON file ravel wrote</li>
    </ul>
    <p>There is no bundler in this pipeline.</p>
  `;
}

// A filter box, which is the reason to have a framework here at all: the list
// re-renders as you type, with no page reload.
function Blog({ posts }) {
  const [query, setQuery] = useState("");
  const needle = query.trim().toLowerCase();
  const matches = needle
    ? posts.filter((post) =>
        `${post.title} ${post.body}`.toLowerCase().includes(needle),
      )
    : posts;

  return html`
    <h1>Blog</h1>
    <label class="filter">
      Filter posts
      <input
        type="search"
        value=${query}
        placeholder="Type to filter..."
        onInput=${(e) => setQuery(e.currentTarget.value)}
      />
    </label>
    <p class="count">
      ${matches.length} of ${posts.length} ${posts.length === 1 ? "post" : "posts"}
    </p>
    ${matches.map((post) => html`<${PostCard} post=${post} />`)}
    ${matches.length === 0 && html`<p>Nothing matches “${query}”.</p>`}
  `;
}

function Post({ post }) {
  if (!post) {
    return html`
      <h1>Not found</h1>
      <p>No post with that name.</p>
      <p><a href="#/blog">Back to blog</a></p>
    `;
  }
  return html`
    <h1>${post.title}</h1>
    <time>${post.date}</time>
    <p>${post.body}</p>
    <p><a href="#/blog">Back to blog</a></p>
  `;
}

function App() {
  const route = useHashRoute();
  const { status, posts, error } = usePosts();

  let view;
  if (status === "loading") {
    view = html`<p>Loading…</p>`;
  } else if (status === "failed") {
    view = html`<p class="error">Could not load posts: ${error}</p>`;
  } else if (route === "about") {
    view = html`<${About} />`;
  } else if (route === "blog") {
    view = html`<${Blog} posts=${posts} />`;
  } else if (route.startsWith("blog/")) {
    const slug = route.slice("blog/".length);
    view = html`<${Post} post=${posts.find((p) => p.slug === slug)} />`;
  } else {
    view = html`<${Home} posts=${posts} />`;
  }

  const tab = route === "about" || route === "blog" ? route : "home";
  return html`
    <header><${Nav} route=${tab} /></header>
    <main>${view}</main>
    <footer><p>Built with <strong>ravel v${RAVEL_VERSION}</strong></p></footer>
  `;
}

render(html`<${App} />`, document.getElementById("app"));
