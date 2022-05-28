import { useEffect, useRef, useState } from 'react';
import { FPS } from '@zxch3n/fps';
import { Graph2D } from '../../../src/graph2d';

let graphData = createGraphData(1);
let fps = new FPS();
function App() {
  const [show, setShow] = useState(false);
  const [num, setNum] = useState(0);
  const [fpsNum, setFps] = useState<undefined | number>(undefined);
  useEffect(() => {
    const func = async () => {
      for (let num = 100; num < 10000; num += 100) {
        graphData = createGraphData(num);
        fps = new FPS();
        setNum(num);
        setShow(true);
        await new Promise((r) => setTimeout(r, 1000));
        while (fps.fps == null) {
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

function Graph() {
  const ref = useRef(null);
  useEffect(() => {
    const graph = new Graph2D();

    graph.data(graphData);
    graph.render();
    fps.start();
    return () => {
      graph.destroy();
    };
  }, []);
  return <div ref={ref} id="container"></div>;
}

function createGraphData(N: number): GraphData {
  return {
    nodes: [...Array(N).keys()].map((i) => ({ id: i + '' })),
    edges: [],
  };
}

export default App;
