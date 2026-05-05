export function Layout(props: { title: string; children: string }) {
  return (
    <html lang="en">
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{props.title}</title>
        <link rel="stylesheet" href="style.css" />
        <base href="/ravel/" />
      </head>
      <body>
        <header>
          <nav>
            <a href="./">Home</a> | <a href="about">About</a> | <a href="blog">Blog</a>
          </nav>
        </header>
        <main>{props.children}</main>
        <footer>
          <p>Built with <strong>Ravel v{ravel.version}</strong></p>
        </footer>
      </body>
    </html>
  );
}

export function PostCard(props: { slug: string; title: string; date: string; excerpt: string }) {
  return (
    <article class="post-card">
      <h2><a href={"blog/" + props.slug + ".html"}>{props.title}</a></h2>
      <time>{props.date}</time>
      <p>{props.excerpt}</p>
    </article>
  );
}