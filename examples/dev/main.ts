// TypeScript, served to the browser as JavaScript. ravel strips the types on
// the way out; there is no build step and no bundler.

// A bare specifier. The browser only knows what to do with it because of the
// import map, and the "browser" condition is what picks browser.js.
import { hello, picked } from "greet";

// A published subpath, which itself relative-imports inside the package.
import { shout } from "greet/loud";

// A scoped package whose exports are a pattern.
import { area } from "@acme/shapes/circle";

// A local module, imported without its extension.
import { title } from "./util";

const lines: string[] = [
  `${title}`,
  `greet          -> ${hello("ravel")}`,
  `condition      -> ${picked}`,
  `greet/loud     -> ${shout("ravel")}`,
  `@acme/shapes   -> circle r=2 is ${area(2).toFixed(2)}`,
];

document.getElementById("out")!.textContent = lines.join("\n");
