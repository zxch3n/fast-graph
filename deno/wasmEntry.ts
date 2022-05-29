import initWasm, { build_a_tree, initThreadPool } from '../wasm_dist/wasm.js';

export async function init(threadNum = navigator.hardwareConcurrency) {
  await initWasm();
  await initThreadPool(threadNum);
}

export function findInside(
  inputCoords: Float64Array,
  target: [number, number],
): number {
  const _target = new Float64Array(target);
  return build_a_tree(inputCoords, _target);
}
