# ts-rust-parallel-boilerplate

- This template set up a basic env for web parallel in Rust
- It is based on [wasm-bindgen-rayon](https://github.com/GoogleChromeLabs/wasm-bindgen-rayon). You can use [`rayon`](https://docs.rs/rayon/latest/rayon/) to do parallelism.
- Write Rust code in `./wasm` folder. Run `yarn build:wasm` to make a wasm build.
  - The built output will be in `./wasm_dist`. It includes the typescript type files.
  - To use the Rust code

```ts
import initWasm, { sum_of_squares, initThreadPool } from '../wasm_dist/wasm';

export async function init() {
  await initWasm();
  await initThreadPool(navigator.hardwareConcurrency || 1);
}
```

- Use `import instance from 'comlink:./worker'` to import a worker instance
  - All exported function from worker can be directly used with type annotation

## Usage

1. Press `Use this template` to create your project
2. Clone from your repo
3. Run `node scripts/init.js` to update `package.json` and `README.md`
