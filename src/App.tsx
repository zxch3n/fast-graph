import { useCallback, useState } from 'react';

import { init, heavy_calc } from './wasm';

const promise = init();
function App() {
  const [running, setRunning] = useState(false);
  const callback = useCallback(async () => {
    await promise;
    setRunning(true);
    await bench('single thread calc', async () => {
      await heavy_calc(false);
    });

    await bench('multi-thread calc', async () => {
      await heavy_calc(true);
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
