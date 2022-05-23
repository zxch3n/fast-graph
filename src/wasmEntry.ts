import initWasm, {
  build_a_tree,
  initThreadPool,
  ForceGraph2D,
} from '../wasm_dist/wasm';

const initPromise = initWasm();
export async function init(threadNum = navigator.hardwareConcurrency) {
  await initPromise;
  await initThreadPool(threadNum);
}

export function findInside(
  inputCoords: Float64Array,
  target: [number, number],
): number {
  const _target = new Float64Array(target);
  return build_a_tree(inputCoords, _target);
}

export class Graph2D {
  positions: Float64Array | undefined;
  private graph: ForceGraph2D | undefined;
  private memory: WebAssembly.Memory | undefined;
  private destroyed = false;
  constructor(private num: number) {}

  async init() {
    if (this.destroyed) {
      return;
    }

    const { memory } = await initPromise;
    this.memory = memory;
    this.graph = ForceGraph2D.from_random(this.num);
    this.positions = new Float64Array(
      memory.buffer,
      this.graph.get_pos(),
      this.num * 2,
    );
  }

  tick() {
    this.graph!.tick(1);
  }

  dispose() {
    this.graph?.free();
    this.destroyed = true;
  }
}
