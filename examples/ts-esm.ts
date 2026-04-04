import { add, multiply, VERSION, type MathResult } from "./math-utils.ts";

console.log(`Math utils v${VERSION}`);

const sum: MathResult = add(10, 5);
console.log(`${sum.operation}: ${sum.value}`);

const product: MathResult = multiply(4, 7);
console.log(`${product.operation}: ${product.value}`);
