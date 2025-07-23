import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

export default defineConfig({
  base: `/rust_raytracer/`,
  publicDir: 'public',
  build: {
    sourcemap: true,
    rollupOptions: {
      output: {
        format: 'es',
      }
    }
  },
  worker: {
    format: 'es'
  },
  plugins: [wasm(), topLevelAwait()]
})