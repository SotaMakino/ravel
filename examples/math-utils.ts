export interface MathResult {
  value: number;
  operation: string;
}

export function add(a: number, b: number): MathResult {
  return { value: a + b, operation: "add" };
}

export function multiply(a: number, b: number): MathResult {
  return { value: a * b, operation: "multiply" };
}

export const VERSION: string = "2.0.0";
