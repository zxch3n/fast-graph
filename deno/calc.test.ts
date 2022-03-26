import { expect } from './utils.ts';
import { calcSumOfSquares, init } from '../src/wasm.ts';

await init();
Deno.test({
  name: 'add',
  fn: () => {
    expect(1 + 1).toBe(2);
  },
});

Deno.test({
  name: 'calc',
  fn: async () => {
    expect(await calcSumOfSquares([1, 2, 3])).toBe(14);
    expect(await calcSumOfSquares([3])).toBe(9);
  },
});
