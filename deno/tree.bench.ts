import { findInside, init } from "../src/wasm.ts";

await init(1);
let input = new Float64Array(new Array(2e6).fill(0).map(() => Math.random()));
await run();

async function run() {
  const durations = [] as number[];
  for (let i = 0; i < 20; i++) {
    const start = performance.now();
    await target();
    const duration = performance.now() - start;
    // console.log(duration);
    if (i > 3) {
      durations.push(duration);
    }
    await new Promise((r) => setTimeout(r, 0));
  }

  const mean = durations.reduce((a, b) => a + b) / durations.length;
  const std = Math.sqrt(
    durations.reduce((a, b) => a + b * b, 0) / durations.length - mean * mean,
  );
  console.log("single thread tree insert 1M >> ", mean, "ms +-", std);
}

async function target() {
  await findInside(input, [1, 1]);
}

Deno.exit();
