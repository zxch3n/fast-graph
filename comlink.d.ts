
      declare module "comlink:./wasmEntry" {
        const mod: () => import("comlink").Remote<typeof import("./src/wasmEntry")>
        export default mod
      }

      declare module "comlink:./worker2" {
        const mod: () => import("comlink").Remote<typeof import("./src/worker2")>
        export default mod
      }