import init, {wasm_main, wasm_log_init, GpuRenderer} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model, GpuModel, CpuModel} from "./model.js";

async function main() {
    console.log(`Main:\tstarted`)

    // Load wasm file, run its entry point
    await init();
    wasm_main();
    wasm_log_init();

    const canvas = document.getElementById('screen') as HTMLCanvasElement

    const view = new View(canvas)
    const controller = new Controller(canvas)
    // const model = new CpuModel(view, controller, canvas)
    const gpu_renderer = await GpuRenderer.new(canvas.width, canvas.height)
    const model = new GpuModel(view, controller, canvas, gpu_renderer)
    controller.set_model(model)
}
main()