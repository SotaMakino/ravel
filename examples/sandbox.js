// Test fs sandbox - path traversal should be denied

console.log("Testing fs sandbox security...");

// This should work - file in the same directory
console.log("fs.js exists:", fs.exists("fs.js"));

// This should fail - trying to read /etc/passwd.
// readFile is async now, so the denial arrives as a rejected promise.
try {
    console.log("Attempting to read /etc/passwd...");
    await fs.readFile("/etc/passwd");
    console.log("SECURITY BUG: should not reach here!");
} catch (e) {
    console.log("Blocked /etc/passwd: permission denied");
}

// This should fail - trying to read outside root via traversal
try {
    console.log("Attempting to read ../Cargo.toml...");
    await fs.readFile("../Cargo.toml");
    console.log("SECURITY BUG: should not reach here!");
} catch (e) {
    console.log("Blocked ../Cargo.toml: permission denied");
}

// The sync variant enforces the same sandbox, and throws directly.
try {
    fs.readFileSync("/etc/passwd");
    console.log("SECURITY BUG: should not reach here!");
} catch (e) {
    console.log("Blocked /etc/passwd (sync): permission denied");
}

// exists returns false for paths outside root (does not leak info)
console.log("/etc/shadow exists:", fs.exists("/etc/shadow"));
console.log("../.env exists:", fs.exists("../.env"));

console.log("Sandbox security tests passed!");
