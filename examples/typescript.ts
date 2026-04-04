interface User {
  name: string;
  age: number;
}

function greet(user: User): string {
  return `Hello, ${user.name}! You are ${user.age} years old.`;
}

const alice: User = { name: "Alice", age: 30 };
console.log(greet(alice));

enum Color {
  Red,
  Green,
  Blue,
}

const favorite: Color = Color.Green;
console.log(`Favorite color: ${Color[favorite]}`);

class Counter {
  count: number;

  constructor(initial: number) {
    this.count = initial;
  }

  increment(): number {
    this.count++;
    return this.count;
  }
}

const counter: Counter = new Counter(0);
counter.increment();
counter.increment();
counter.increment();
console.log(`Counter: ${counter.count}`);

type Result = {
  success: boolean;
  value?: number;
};

function divide(a: number, b: number): Result {
  if (b === 0) {
    return { success: false };
  }
  return { success: true, value: a / b };
}

console.log(divide(10, 2));
console.log(divide(10, 0));
