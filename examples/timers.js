// Timer features: setTimeout, setInterval, clearTimeout, clearInterval

console.log("=== setTimeout ===");
const start = Date.now();

setTimeout(() => {
  const elapsed = Date.now() - start;
  console.log("1. Fired after ~" + elapsed + "ms");
}, 100);

setTimeout(() => {
  const elapsed = Date.now() - start;
  console.log("2. Fired after ~" + elapsed + "ms");
}, 200);

console.log("=== clearTimeout ===");
const cancelledId = setTimeout(() => {
  console.log("This should NOT print");
}, 50);

setTimeout(() => {
  console.log("Cancelled timer was cleared");
}, 100);

clearTimeout(cancelledId);

console.log("=== setInterval ===");
let count = 0;
const intervalId = setInterval(() => {
  count++;
  console.log("Interval tick #" + count);
  if (count >= 3) {
    clearInterval(intervalId);
    console.log("Interval cleared after 3 ticks");
  }
}, 100);

console.log("=== Nested Timers ===");
setTimeout(() => {
  console.log("Outer timer fired");
  setTimeout(() => {
    console.log("Inner timer fired");
  }, 50);
}, 150);

console.log("All timers scheduled, waiting...");
