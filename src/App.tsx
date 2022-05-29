import { useEffect, useMemo, useRef } from 'react';

import { Graph2D } from './graph2d';
import { genGraph } from './utils';

// init(1);
const W = 800;
const H = 800;
const graphData = genGraph(2000, 2000);
function App() {
  const graph = useMemo(() => new Graph2D(), []);
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const ctx = canvas.current!.getContext('2d')!;

    graph.init().then(() => {
      graph.setData(graphData);
      draw();
    });
    function draw() {
      graph.tick(1);
      graph.draw(ctx);
      requestAnimationFrame(draw);
    }

    return () => {
      graph.dispose();
    };
  }, []);

  return (
    <div className="App">
      <canvas
        width={W}
        height={H}
        ref={canvas}
        style={{ width: W / 2, height: H / 2 }}
      />
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
