import init, {wasm_main, wasm_log_init, GpuRenderer} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {GpuModel, CpuModel} from "./model.js";

async function main() {
    console.log(`Main:\tstarted`)

    // Load wasm file, run its entry point
    await init();
    wasm_main();
    wasm_log_init();

    const canvas = document.getElementById('screen') as HTMLCanvasElement

    const view = new View(canvas)
    const controller = new Controller(canvas)

    // TODO: UI CPU/GPU switch
        // TODO: wasm: Animator - shared by CPU/GPU
    const USE_GPU = true
    if (USE_GPU) {
        const gpu_renderer = await GpuRenderer.new(canvas.width, canvas.height)
        const gpu_model = new GpuModel(view, controller, canvas, gpu_renderer)
        controller.set_model(gpu_model)
    } else {
        const cpu_model = await CpuModel.create(view, controller, canvas)
        controller.set_model(cpu_model)
    }
}
main()