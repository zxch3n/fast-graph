import instance from "comlink:./wasmEntry";
export const { calcSumOfSquares, init, findInside, heavy_calc, js_parallel } =
  instance();
