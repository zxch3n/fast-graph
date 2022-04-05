import { heavy_calc, init } from "../src/wasm.ts";
import { bench } from "./utils.ts";

await init();

await bench("single thread calc", async () => {
  await heavy_calc(false);
});

await bench("multi-thread calc", async () => {
  await heavy_calc(true);
});

Deno.exit();
