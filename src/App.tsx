import { useCallback, useState } from 'react';

import { findInside, init } from './wasm';

const promise = init(1);
const input = new Float64Array(new Array(1e4).fill(0).map(() => Math.random()));
function App() {
  const [running, setRunning] = useState(false);
  const callback = useCallback(async () => {
    setRunning(true);
    await promise;
    await target();
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

async function run() {
  const testStart = performance.now();
  const durations = [] as number[];
  for (let i = 0; i < 20; i++) {
    const start = performance.now();
    await target();
    const duration = performance.now() - start;
    // console.log(duration);
    if (i > 3) {
      durations.push(duration);
    }

    await new Promise((r) => setTimeout(r, 100));
    if (performance.now() - testStart > 5_000) {
      break;
    }
  }

  const mean = durations.reduce((a, b) => a + b) / durations.length;
  const std = Math.sqrt(
    durations.reduce((a, b) => a + b * b, 0) / durations.length - mean * mean,
  );
  console.log('4 threads tree insert 1k >> ', mean, 'ms +-', std);
}

async function target() {
  await findInside(input, [1, 1]);
}
