import initWasm, {
  build_a_tree,
  initThreadPool,
  ForceGraph2D,
} from '../wasm_dist/wasm';
import { DefaultNodeValue, GraphData, Node } from './type';

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
  public graph: ForceGraph2D | undefined;
  private memory: WebAssembly.Memory | undefined;
  private destroyed = false;
  private idToNode: Map<number | string, number> = new Map();
  private data: GraphData | undefined;
  constructor() {}

  async init() {
    if (this.destroyed) {
      return;
    }

    const { memory } = await initPromise;
    this.memory = memory;
  }

  draw(ctx: CanvasRenderingContext2D) {
    if (!this.data || !this.positions) {
      return;
    }

    ctx.save();
    const { nodes, links } = this.data;
    const { positions } = this;
    ctx.clearRect(0, 0, ctx.canvas.width, ctx.canvas.height);
    ctx.translate(ctx.canvas.width / 2, ctx.canvas.height / 2);
    ctx.scale(0.5, 0.5);
    ctx.fillStyle = '#000';
    ctx.globalAlpha = 0.5;
    ctx.strokeStyle = '#f00';
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      const x = positions[i * 2];
      const y = positions[i * 2 + 1];
      ctx.beginPath();
      ctx.moveTo(x, y);
      ctx.arc(x, y, node.size || DefaultNodeValue.Size, 0, Math.PI * 2);
      ctx.closePath();
      ctx.fill();
    }

    for (let i = 0; i < links.length; i++) {
      const link = links[i];
      const from = this.idToNode.get(link.from)!;
      const to = this.idToNode.get(link.to)!;
      ctx.beginPath();
      ctx.moveTo(positions[from * 2], positions[from * 2 + 1]);
      ctx.lineTo(positions[to * 2], positions[to * 2 + 1]);
      ctx.closePath();
      ctx.stroke();
    }
    ctx.restore();
  }

  setData(data: GraphData) {
    this.data = data;
    this.updateIdToNode(data.nodes);
    const links = [];
    for (const link of data.links) {
      links.push(this.idToNode.get(link.from)!, this.idToNode.get(link.to)!);
    }

    this.graph = ForceGraph2D.build_graph(
      data.nodes.length,
      new Uint32Array(links),
    );
    this.positions = new Float64Array(
      this.memory!.buffer,
      this.graph!.get_pos(),
      data.nodes.length * 2,
    );

    this.graph.add_n_body_force();
    this.graph.add_center_force();
  }

  private updateIdToNode(nodes: Node[]) {
    this.idToNode.clear();
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      this.idToNode.set(node.id ?? i, i);
    }
  }

  tick(times = 1, changed = false) {
    this.graph!.tick(times, changed);
  }

  dispose() {
    this.graph?.free();
    this.destroyed = true;
  }
}
