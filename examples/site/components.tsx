// The shell ravel renders at build time. Everything inside #app is drawn by
// Preact in the browser -- see app.js.

export function Shell(props: {
  title: string;
  base: string;
  version: string;
  importMap: string;
}) {
  return (
    <html lang="en" data-ravel-version={props.version}>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1" />
        <title>{props.title}</title>
        <link rel="stylesheet" href={props.base + "style.css"} />
        <base href={props.base} />
        {/*
          An import map must be inline: browsers do not load external ones. Its
          body is raw text to the parser, so note.raw keeps the quotes intact
          instead of escaping them into &quot;. This map is what lets app.js
          write `from "preact"` with no bundler in the pipeline.
        */}
        <script type="importmap">{note.raw(props.importMap)}</script>
      </head>
      <body>
        <div id="app">
          <noscript>
            <p>
              This page is rendered in the browser, so it needs JavaScript
              enabled.
            </p>
          </noscript>
        </div>
        <script type="module" src="./app.js"></script>
      </body>
    </html>
  );
}
