// Module resolution: node_modules, exports, imports, and conditions.
//
// Everything imported below lives beside this file. Look at
// examples/packages/ to see the tree the resolver is walking.

// A bare specifier. The resolver walks up from this file looking for
// node_modules/greet, then asks that package's "exports" what "." means.
import { hello, picked } from "greet";

// A subpath the package chose to publish.
import { shout } from "greet/loud";

// A scoped package whose exports are a pattern: "./*" -> "./src/*.js".
import { area as circleArea } from "@acme/shapes/circle";
import { area as squareArea } from "@acme/shapes/square";

// A "#" specifier, resolved through this package's own "imports" map.
import { appName } from "#config";

// "imports" entries can point at a real package, not just a file.
import { hello as viaAlias } from "#greeter";

console.log("=== bare specifier ===");
console.log(hello(appName));
console.log("condition picked:", picked);

console.log("=== published subpath ===");
console.log(shout(appName));

console.log("=== pattern exports ===");
console.log("circle r=2:", circleArea(2).toFixed(2));
console.log("square s=3:", squareArea(3));

console.log("=== #imports ===");
console.log("#config gave:", appName);
console.log("#greeter gave:", viaAlias("again"));

console.log("=== encapsulation ===");
// greet/internal.js is a real file, but "exports" does not name it, so the
// package has not published it and the import fails.
try {
  await import("greet/internal.js");
  console.log("imported internal.js -- exports did not encapsulate it");
} catch {
  console.log("greet/internal.js is not exported, so it cannot be imported");
}
