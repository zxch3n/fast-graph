import { useEffect, useRef, useState } from 'react';
import { FPS } from '@zxch3n/fps';
import { Graph2D } from '../../../src/graph2d';
import { GraphData } from '../../../src/type';

let graphData = genGraph(1, 1);
let fps = new FPS();
function App() {
  const [show, setShow] = useState(false);
  const [num, setNum] = useState(0);
  const [fpsNum, setFps] = useState<undefined | number>(undefined);
  useEffect(() => {
    const func = async () => {
      for (let num = 4000; num < 20000; num += 1000) {
        graphData = genGraph(num, num);
        fps = new FPS();
        setNum(num);
        setShow(true);
        await new Promise((r) => setTimeout(r, 1000));
        while (fps.fps == null) {
          fps.start();
          await new Promise((r) => setTimeout(r, 1000));
        }
        if (fps.fps != null && fps.fps < 50) {
          console.log('MAX NUM', num);
          setFps(fps.fps);
          break;
        }

        if (fps.fps) {
          setFps(fps.fps);
        }
        setShow(false);
        await new Promise((r) => setTimeout(r, 100));
      }
    };
    func();
  }, []);
  return (
    <div className="App">
      <p style={{ textAlign: 'center' }}>{num} Nodes</p>
      <p style={{ textAlign: 'center' }}>fps {fpsNum}</p>
      {show && <Graph />}
    </div>
  );
}

const W = 800;
const H = 800;
function Graph() {
  const canvas = useRef<HTMLCanvasElement>(null);
  useEffect(() => {
    const ctx = canvas.current!.getContext('2d')!;
    const graph = new Graph2D();
    let stop = false;
    graph.init().then(() => {
      graph.setData(graphData);
      fps.start();
      draw();
    });
    function draw() {
      if (stop) {
        return;
      }
      graph.tick(1);
      graph.draw(ctx);
      requestAnimationFrame(draw);
    }

    return () => {
      stop = true;
      graph.dispose();
    };
  }, []);
  return (
    <div id="container">
      <canvas
        width={W}
        height={H}
        ref={canvas}
        style={{ width: W / 2, height: H / 2 }}
      />
    </div>
  );
}

export function genGraph(node_num: number, edge_num: number): GraphData {
  return {
    nodes: [...Array(node_num)].fill({}),
    links: [...Array(edge_num)].map((_, i) => {
      const from = i;
      let to = (Math.random() * node_num) | 0;
      if (to === from) {
        to = (from + 1) % node_num;
      }
      return {
        from,
        to,
      };
    }),
  };
}

export default App;
