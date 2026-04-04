// JSX rendering with note() function
// JSX elements are transpiled to note() calls and rendered to HTML strings

console.log("=== Simple Elements ===");
const heading = <h1>Hello, Ravel!</h1>;
console.log(heading);

console.log("=== With Attributes ===");
const link = <a href="https://example.com" target="_blank">Click here</a>;
console.log(link);

console.log("=== Nested Elements ===");
const card = (
  <div class="card">
    <h2>Card Title</h2>
    <p>This is a card with some content.</p>
  </div>
);
console.log(card);

console.log("=== Self-Closing Tags ===");
const br = <br />;
const img = <img src="logo.png" alt="Logo" />;
console.log(br);
console.log(img);

console.log("=== Fragments ===");
const items = (
  <>
    <li>Item 1</li>
    <li>Item 2</li>
    <li>Item 3</li>
  </>
);
console.log(items);

console.log("=== Function Components ===");
function Badge(props) {
  return <span class="badge">{props.text}</span>;
}

const badge = <Badge text="New" />;
console.log(badge);

console.log("=== Full Page ===");
function Layout(props) {
  return (
    <html>
      <head>
        <title>{props.title}</title>
      </head>
      <body>
        <header>
          <h1>{props.title}</h1>
        </header>
        <main>{props.children}</main>
        <footer>
          <p>Footer content</p>
        </footer>
      </body>
    </html>
  );
}

const page = (
  <Layout title="My App">
    <p>Welcome to my app!</p>
  </Layout>
);
console.log(page);

console.log("=== Done ===");
