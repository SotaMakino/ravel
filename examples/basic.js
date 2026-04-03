// Basic JavaScript features

console.log("=== Variables ===");
let x = 10;
const y = 20;
console.log("x =", x);
console.log("y =", y);
console.log("x + y =", x + y);

console.log("=== Functions ===");
function add(a, b) {
  return a + b;
}
console.log("add(3, 4) =", add(3, 4));

const multiply = (a, b) => a * b;
console.log("multiply(3, 4) =", multiply(3, 4));

console.log("=== Control Flow ===");
if (x > 5) {
  console.log("x is greater than 5");
} else {
  console.log("x is not greater than 5");
}

console.log("=== Loops ===");
for (let i = 0; i < 3; i++) {
  console.log("i =", i);
}

console.log("=== Arrays ===");
const arr = [1, 2, 3, 4, 5];
console.log("arr =", arr);
console.log("arr[0] =", arr[0]);
console.log("arr.length =", arr.length);

console.log("=== Objects ===");
const person = { name: "Alice", age: 30 };
console.log("person =", person);
console.log("person.name =", person.name);

console.log("=== Math ===");
console.log("Math.PI =", Math.PI);
console.log("Math.sqrt(144) =", Math.sqrt(144));
console.log("Math.max(1, 5, 3) =", Math.max(1, 5, 3));

console.log("=== JSON ===");
const obj = { x: 1, y: 2 };
const json = JSON.stringify(obj);
console.log("stringified:", json);
console.log("parsed:", JSON.parse(json));

console.log("=== Done ===");
