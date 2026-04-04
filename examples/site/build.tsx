function Layout(props: { title: string; children: string }) {
  return (
    <html>
      <head>
        <meta charset="utf-8" />
        <title>{props.title}</title>
      </head>
      <body>{props.children}</body>
    </html>
  );
}

const pages = [
  { path: "index.html", title: "Home", body: <><h1>Welcome</h1><p>A minimal ravel site.</p></> },
  { path: "about.html", title: "About", body: <><h1>About</h1><p>Built with ravel.</p></> },
];

for (const page of pages) {
  const html = "<!DOCTYPE html>" + <Layout title={page.title}>{page.body}</Layout>;
  const bytes = new Uint8Array(html.length);
  for (var i = 0; i < html.length; i++) {
    bytes[i] = html.charCodeAt(i);
  }
  fs.writeFile("dist/" + page.path, bytes);
  console.log("wrote dist/" + page.path);
}

console.log("done - " + pages.length + " pages (ravel " + ravel.version + ")");
