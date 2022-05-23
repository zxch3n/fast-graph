import type { Remote } from 'comlink';
import { useEffect, useMemo, useRef } from 'react';

import { Graph2D, init } from './wasmEntry';

// init(1);
const W = 500;
const H = 500;
function App() {
  const graph = useMemo(() => new Graph2D(100), []);
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const ctx = canvas.current!.getContext('2d')!;

    async function draw() {
      if (!graph) {
        return;
      }
      await graph.init();
      let positions = graph.positions;
      if (!positions) {
        return;
      }

      ctx.clearRect(0, 0, W, H);
      ctx.translate(W / 2, H / 2);
      ctx.fillStyle = '#000';
      let n = positions.length;
      for (let i = 0; i < n; i += 2) {
        const x = positions[i];
        const y = positions[i + 1];
        // draw a circle
        ctx.beginPath();
        ctx.arc(x, y, 5, 0, 2 * Math.PI);
        ctx.closePath();
        ctx.fill();
      }

      ctx.translate(-W / 2, -H / 2);
      graph.tick();
      requestAnimationFrame(draw);
    }

    draw();

    return () => {
      graph.dispose();
    };
  }, []);

  return (
    <div className="App">
      <canvas width={W} height={H} ref={canvas} />
    </div>
  );
}

export default App;

export async function bench(name: string, fn: () => void | Promise<void>) {
  const start = performance.now();
  await fn();
  const end = performance.now();
  console.log(`${name}: ${end - start}ms`);
}
