import { findInside, init } from "../src/wasm.ts";

await init(4);
let input = new Float64Array(new Array(1e3).fill(0).map(() => Math.random()));
await run();

async function run() {
  const testStart = performance.now();
  const durations = [] as number[];
  for (let i = 0; i < 20; i++) {
    const start = performance.now();
    await target();
    const duration = performance.now() - start;
    // console.log(duration);
    if (i > 3) {
      durations.push(duration);
    }

    await new Promise((r) => setTimeout(r, 100));
    if (performance.now() - testStart > 5_000) {
      break;
    }
  }

  const mean = durations.reduce((a, b) => a + b) / durations.length;
  const std = Math.sqrt(
    durations.reduce((a, b) => a + b * b, 0) / durations.length - mean * mean,
  );
  console.log("4 threads tree insert 1k >> ", mean, "ms +-", std);
}

async function target() {
  await findInside(input, [1, 1]);
}

Deno.exit();
