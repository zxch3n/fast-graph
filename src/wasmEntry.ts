import initWasm, {
  build_a_tree,
  initThreadPool,
  sum_of_squares,
} from "../wasm_dist/wasm";

export { heavy_calc } from "../wasm_dist/wasm";

export async function init(threadNum = navigator.hardwareConcurrency) {
  await initWasm();
  await initThreadPool(threadNum);
}

export async function calcSumOfSquares(ints: number[]) {
  return sum_of_squares(new Int32Array([...ints]));
}

export function findInside(
  inputCoords: Float64Array,
  target: [number, number],
): number {
  const _target = new Float64Array(target);
  return build_a_tree(inputCoords, _target);
}
