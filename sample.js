// Sample JavaScript for ravel

// Variables
let x = 10;
let y = 20;
let sum = x + y;

console.log("=== Variables ===");
console.log(x);
console.log(y);
console.log(sum);

// Control flow
console.log("=== Control Flow ===");
if (sum > 25) {
  console.log("sum is greater than 25");
} else {
  console.log("sum is not greater than 25");
}

// Loops
console.log("=== While Loop ===");
let i = 0;
while (i < 5) {
  console.log(i);
  i = i + 1;
}

// For loop
console.log("=== For Loop ===");
for (let j = 0; j < 3; j = j + 1) {
  console.log(j);
}

// Functions
console.log("=== Functions ===");
function add(a, b) {
  return a + b;
}

function greet(name) {
  return "Hello, " + name + "!";
}

console.log(add(5, 7));
console.log(greet("ravel"));

// Objects
console.log("=== Objects ===");
let person = { name: "Alice", age: 30 };
console.log(person);

// Arrays
console.log("=== Arrays ===");
let arr = [1, 2, 3];
console.log(arr);

// String concatenation
console.log("=== String Concat ===");
let result = add(100, 200);
console.log("The result is: " + result);
