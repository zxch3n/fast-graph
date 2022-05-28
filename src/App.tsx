import { useEffect, useMemo, useRef } from 'react';

import { Graph2D } from './graph2d';

// init(1);
const W = 500;
const H = 500;
function App() {
  const graph = useMemo(() => new Graph2D(100), []);
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const ctx = canvas.current!.getContext('2d')!;

    graph.init().then(() => {
      draw();
      graph.graph?.add_center_force();
      graph.graph?.add_n_body_force('nbody');
    });
    function draw() {
      let positions = graph.positions;
      if (!positions) {
        return;
      }

      graph.tick(1);
      ctx.clearRect(0, 0, W, H);
      ctx.save();
      ctx.translate(W / 2, H / 2);
      ctx.fillStyle = '#000';
      ctx.scale(0.1, 0.1);
      let n = positions.length;
      for (let i = 0; i < n; i += 2) {
        const x = positions[i];
        const y = positions[i + 1];
        // draw a circle
        ctx.beginPath();
        ctx.arc(x, y, 20, 0, 2 * Math.PI);
        ctx.closePath();
        ctx.fill();
      }

      ctx.restore();
      requestAnimationFrame(draw);
    }

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
