import { findInside, init } from "../src/wasm.ts";
import { expect } from "./utils.ts";

await init();
Deno.test({
  name: "tree",
  fn: async () => {
    expect(await findInside([[0, 0], [1, 1], [2, 2]], [1, 1])).toBe(1);
    expect(await findInside([[0, 0], [1, 1], [2, 2]], [0, 0])).toBe(0);
    expect(await findInside([[0, 0], [1, 1], [2, 2]], [1.9, 1.8])).toBe(2);
  },
});
