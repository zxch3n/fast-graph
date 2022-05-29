import { findInside, init } from "./wasmEntry.ts";

await init(4);
let input = new Float64Array(new Array(1e3).fill(0).map(() => Math.random()));
// @ts-ignore
Deno.bench("build tree with 1K nodes by 4 threads", () => {
  findInside(input, [1, 1]);
});
