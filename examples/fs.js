// Test fs module
console.log("Testing fs module...");

// Test fs.exists
console.log("sandbox.js exists:", fs.exists("sandbox.js"));
console.log("nonexistent.txt exists:", fs.exists("nonexistent.txt"));

// fs.readFile is async: it returns a promise, and the read runs on Tokio.
const pending = fs.readFile("sandbox.js");
console.log("readFile returns:", pending.constructor.name);

const data = await pending;
console.log("readFile type:", typeof data);
console.log("readFile length:", data.length);
console.log(
  "First 50 bytes as string:",
  new TextDecoder().decode(data.slice(0, 50)),
);

// fs.writeFile is async too.
await fs.writeFile("output.txt", "Hello from Ravel!");
console.log("Wrote to output.txt");

// Verify the written file
const readData = await fs.readFile("output.txt");
console.log("Read back:", new TextDecoder().decode(readData));

// Reads run concurrently: all three are issued before any completes.
const many = await Promise.all([
  fs.readFile("output.txt"),
  fs.readFile("output.txt"),
  fs.readFile("output.txt"),
]);
console.log("Concurrent reads:", many.length);

// Sync variants exist for scripts that read top to bottom.
fs.writeFileSync("output-sync.txt", "written synchronously");
console.log(
  "Sync read back:",
  new TextDecoder().decode(fs.readFileSync("output-sync.txt")),
);

// Test __dirname and __filename
console.log("__filename:", __filename);
console.log("__dirname:", __dirname);

console.log("All tests passed!");
