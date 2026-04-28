export type GreetOptions = {
  name?: string;
  count?: number;
};

export function greet(options: GreetOptions): void {
  const name = options.name ?? "World";
  const count = options.count ?? 1;

  for (let index = 0; index < count; index += 1) {
    console.log(`Hello, ${name}!`);
  }
}
