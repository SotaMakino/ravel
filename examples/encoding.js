// TextEncoder / TextDecoder, console streams, and process globals.

// TextEncoder produces UTF-8 bytes; multi-byte characters count as bytes.
const encoder = new TextEncoder();
console.log("encoding:", encoder.encoding);
console.log("ascii bytes:", encoder.encode("abc").length);
console.log("multibyte bytes:", encoder.encode("日本").length);

// TextDecoder reads them back.
const decoder = new TextDecoder();
console.log("round trip:", decoder.decode(encoder.encode("héllo 世界")));

// Only utf-8 is supported; anything else throws rather than mis-decoding.
try {
  new TextDecoder("latin1");
} catch (e) {
  console.log("rejected encoding:", e.name);
}

// fs.writeFile takes a string directly, or bytes for binary output.
fs.writeFile("encoding-out.txt", "written from a string");
console.log("string write:", decoder.decode(fs.readFile("encoding-out.txt")));

fs.writeFile("encoding-out.bin", encoder.encode("written from bytes"));
console.log("bytes write:", decoder.decode(fs.readFile("encoding-out.bin")));

// console.log/info/debug go to stdout; warn/error go to stderr.
console.info("info goes to stdout");
console.debug("debug goes to stdout");
console.warn("warn goes to stderr");
console.error("error goes to stderr");

// process.argv is [execPath, scriptPath, ...userArgs].
console.log("argv entries:", process.argv.length >= 2);
console.log("script is argv[1]:", process.argv[1].endsWith("encoding.js"));

// process.exit(code) stops the script immediately.
console.log("done");
