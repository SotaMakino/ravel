// The event loop: one loop, waiting on timers and I/O at the same time.

const t0 = Date.now();
const log = (msg) => console.log(`~${Date.now() - t0}ms  ${msg}`);
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

log("=== a timer and a top-level await ===");

// Scheduled before the awaits below, and it still fires on time. The loop is
// waiting on this deadline and on the module's await together, so neither one
// has to finish before the other can make progress.
setTimeout(() => log("timer: 40ms, fired while the module was parked"), 40);

await sleep(20);
log("await: slept 20ms");
await sleep(40);
log("await: slept another 40ms");

log("=== a read and a timer, issued together ===");

// Both are handed to the same loop, and whichever is ready first is served
// first. Here that is the read: a local file beats a 30ms deadline.
setTimeout(() => log("timer: 30ms deadline, reached second"), 30);

const bytes = await fs.readFile("event-loop.js");
const firstLine = new TextDecoder().decode(bytes).split("\n")[0];
log(`read: back first, first line is ${JSON.stringify(firstLine)}`);

// Long enough for the timer above to come due.
await sleep(40);

log("=== setInterval ===");

// The loop sleeps until each deadline. Nothing here is polled.
let ticks = 0;
await new Promise((resolve) => {
  const id = setInterval(() => {
    ticks += 1;
    log(`interval: tick ${ticks}`);
    if (ticks === 3) {
      clearInterval(id);
      resolve();
    }
  }, 20);
});

log("done: nothing left to wait for, so the loop exits");
