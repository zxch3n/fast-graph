import { useCallback, useState } from 'react';

import { init, heavy_calc, findInside } from './wasm';

const promise = init();
const data = new Float64Array(new Array(1e4).fill(0).map((_) => Math.random()));
function App() {
  const [running, setRunning] = useState(false);
  const callback = useCallback(async () => {
    await promise;
    setRunning(true);
    await bench('single thread calc', async () => {
      await findInside(data, [0.5, 0.5]);
    });

    setRunning(false);
  }, []);

  return (
    <div className="App">
      <header className="App-header">
        <p>Hello Vite + React!</p>
        <p>
          <button type="button" onClick={callback}>
            {running ? '...' : 'RUN'}
          </button>
        </p>
        <p>
          Edit <code>App.tsx</code> and save to test HMR updates.
        </p>
        <p>
          <a
            className="App-link"
            href="https://reactjs.org"
            target="_blank"
            rel="noopener noreferrer"
          >
            Learn React
          </a>
          {' | '}
          <a
            className="App-link"
            href="https://vitejs.dev/guide/features.html"
            target="_blank"
            rel="noopener noreferrer"
          >
            Vite Docs
          </a>
        </p>
      </header>
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
