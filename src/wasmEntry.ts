import initWasm, {
  build_a_tree,
  initThreadPool,
  sum_of_squares,
} from "../wasm_dist/wasm";

export async function init() {
  await initWasm();
  await initThreadPool(navigator.hardwareConcurrency || 1);
}

export async function calcSumOfSquares(ints: number[]) {
  return sum_of_squares(new Int32Array([...ints]));
}

export function findInside(
  inputCoords: [number, number][],
  target: [number, number],
): number {
  const input = new Float64Array(inputCoords.flat());
  const _target = new Float64Array(target);
  return build_a_tree(input, _target);
}
