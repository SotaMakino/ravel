import { add, PI } from "./math-utils.js";
import greet from "./math-utils.js";

console.log("=== ESM Imports ===");
console.log("add(5, 3) =", add(5, 3));
console.log("PI =", PI);
console.log(greet("World"));
console.log("=== ESM Done ===");
