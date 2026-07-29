// Demonstrates error reporting: stack traces, unhandled rejections, and
// promise callbacks running to completion.

// Caught errors report name, message, and stack via the Error object.
function inner() {
  null.x;
}

function outer() {
  inner();
}

try {
  outer();
} catch (e) {
  console.log("caught:", e.name + ":", e.message);
  console.log("has stack:", e.stack.length > 0);
}

// Microtasks run: the module body settles before the process exits.
Promise.resolve("resolved").then((v) => console.log("microtask:", v));

// A rejection that gets a handler is not reported.
Promise.reject(new Error("handled")).catch((e) =>
  console.log("handled rejection:", e.message),
);

// async/await continuations run too.
async function withAwait() {
  await null;
  console.log("after await");

  try {
    await Promise.reject(new Error("awaited"));
  } catch (e) {
    console.log("caught awaited rejection:", e.message);
  }
}

withAwait();

// A timer callback's errors are reported against user frames only.
setTimeout(() => {
  console.log("timer ran");
}, 1);
