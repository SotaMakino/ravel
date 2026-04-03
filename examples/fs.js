// Test fs module
console.log("Testing fs module...");

// Test fs.exists
console.log("sandbox.js exists:", fs.exists("sandbox.js"));
console.log("nonexistent.txt exists:", fs.exists("nonexistent.txt"));

// Test fs.readFile (returns Uint8Array)
var data = fs.readFile("sandbox.js");
console.log("readFile type:", typeof data);
console.log("readFile length:", data.length);
console.log("First 50 bytes as string:", String.fromCharCode.apply(null, data.slice(0, 50)));

// Test fs.writeFile
var content = "Hello from Ravel!";
var bytes = new Uint8Array(content.length);
for (var i = 0; i < content.length; i++) {
  bytes[i] = content.charCodeAt(i);
}
fs.writeFile("output.txt", bytes);
console.log("Wrote to output.txt");

// Verify the written file
var readData = fs.readFile("output.txt");
console.log("Read back:", String.fromCharCode.apply(null, readData));

// Test __dirname and __filename
console.log("__filename:", __filename);
console.log("__dirname:", __dirname);

console.log("All tests passed!");
