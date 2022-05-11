import initWasm, {
  build_a_tree,
  heavy_calc,
  initThreadPool,
  js_parallel,
  sum_of_squares,
} from "../wasm_dist/wasm.js";

export { heavy_calc };

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
