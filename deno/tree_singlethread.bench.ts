import { findInside, init } from "./wasmEntry.ts";

await init(1);
let input = new Float32Array(new Array(1e5).fill(0).map(() => Math.random()));
// @ts-ignore
Deno.bench("build tree with 100K nodes by 1 thread", () => {
  findInside(input, [1, 1]);
});
