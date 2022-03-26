import initWasm, { sum_of_squares, initThreadPool } from '../wasm_dist/wasm';

export async function init() {
  await initWasm();
  await initThreadPool(navigator.hardwareConcurrency || 1);
}

export async function calcSumOfSquares(ints: number[]) {
  return sum_of_squares(new Int32Array([...ints]));
}
