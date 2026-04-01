// Sample JavaScript for JavaScriptCore backend
// Showcases features the manual backend doesn't support yet

console.log("=== Arrow Functions ===");
const add = (a, b) => a + b;
console.log("5 + 3 =", add(5, 3));

const square = x => x * x;
console.log("square(7) =", square(7));

// Template literals
console.log("=== Template Literals ===");
const name = "ravel";
const version = 2;
console.log(`Running ${name} v${version}`);
console.log(`2 + 2 = ${2 + 2}`);

// Array methods
console.log("=== Array Methods ===");
const nums = [1, 2, 3, 4, 5];
console.log("map:", nums.map(x => x * 2));
console.log("filter:", nums.filter(x => x > 3));
console.log("reduce:", nums.reduce((a, b) => a + b, 0));
console.log("find:", nums.find(x => x === 4));

// Math
console.log("=== Math ===");
console.log("PI:", Math.PI);
console.log("sqrt(144):", Math.sqrt(144));
console.log("max(1, 5, 3):", Math.max(1, 5, 3));
console.log("random:", Math.random());

// Object methods
console.log("=== Object Methods ===");
const person = { first: "Alice", last: "Smith", age: 30 };
console.log("keys:", Object.keys(person));
console.log("values:", Object.values(person));

// Date
console.log("=== Date ===");
console.log("now:", Date.now());
console.log("year:", new Date().getFullYear());

// String methods
console.log("=== String Methods ===");
const str = "hello world";
console.log("toUpperCase:", str.toUpperCase());
console.log("split:", str.split(" "));
console.log("includes:", str.includes("world"));
console.log("substring:", str.substring(0, 5));

// Destructuring
console.log("=== Destructuring ===");
const [a, b, ...rest] = [10, 20, 30, 40, 50];
console.log("a =", a, "b =", b, "rest =", rest);

const { first, age } = person;
console.log("first =", first, "age =", age);

// Classes
console.log("=== Classes ===");
class Animal {
  constructor(name) {
    this.name = name;
  }
  speak() {
    return `${this.name} makes a sound`;
  }
}

class Dog extends Animal {
  speak() {
    return `${this.name} barks`;
  }
}

const dog = new Dog("Rex");
console.log(dog.speak());

// Promises
console.log("=== Promises ===");
const p = new Promise(resolve => {
  setTimeout(() => resolve("done!"), 100);
});
p.then(result => console.log("Promise:", result));

// JSON
console.log("=== JSON ===");
const obj = { x: 1, y: 2 };
const json = JSON.stringify(obj);
console.log("stringified:", json);
console.log("parsed:", JSON.parse(json).x);

// Try/catch
console.log("=== Try/Catch ===");
try {
  JSON.parse("invalid");
} catch (e) {
  console.log("caught:", e.message);
}

// Spread operator
console.log("=== Spread ===");
const arr1 = [1, 2];
const arr2 = [3, 4];
console.log("merged:", [...arr1, ...arr2]);

const defaults = { theme: "dark", lang: "en" };
const user = { theme: "light" };
console.log("merged obj:", { ...defaults, ...user });

console.log("=== All done! ===");
