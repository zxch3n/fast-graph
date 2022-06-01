import { findInside, init } from "./wasmEntry.ts";
import { expect } from "./utils.ts";

await init();
Deno.test({
  name: "tree",
  fn: async () => {
    expect(findInside(new Float32Array([0, 0, 1, 1, 2, 2]), [1, 1])).toBe(
      1,
    );
    expect(findInside(new Float32Array([0, 0, 1, 1, 2, 2]), [0, 0])).toBe(
      0,
    );
    expect(findInside(new Float32Array([0, 0, 1, 1, 2, 2]), [1.9, 1.8]))
      .toBe(2);
  },
});
