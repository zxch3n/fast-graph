export { expect } from "https://deno.land/x/expect@v0.2.9/mod.ts";
export async function bench(
  name: string,
  fn: () => void | Promise<void>,
) {
  const start = performance.now();
  await fn();
  const end = performance.now();
  console.log(`${name}: ${end - start}ms`);
}
