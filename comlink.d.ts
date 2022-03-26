
      declare module "comlink:./wasmEntry" {
        const mod: () => import("comlink").Remote<typeof import("./src/wasmEntry")>
        export default mod
      }