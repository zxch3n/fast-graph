import { init, js_parallel } from "../src/wasm.ts";
import { bench } from "./utils.ts";

await init();

await bench("single thread calc", async () => {
  await js_parallel();
});

Deno.exit();
