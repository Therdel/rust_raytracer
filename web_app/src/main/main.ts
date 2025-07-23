import init, { wasm_main, wasm_log_init } from "../../wasm/pkg/wasm"
import { Controller } from "./controller";
import { CpuModel } from "./cpu_model";
import { GpuModel } from "./gpu_model";
import { Model } from "./model";
import { View } from "./view";

async function start_render_loop(model: Model) {
    const renderLoop = async () => {
        await model.render()
        requestAnimationFrame(renderLoop)
    }
    await renderLoop()
}

async function main() {
    console.log(`Main:\tstarted`)

    // Load wasm file, run its entry point
    await init();
    wasm_main();
    wasm_log_init();

    const canvas = document.getElementById('screen') as HTMLCanvasElement
    const canvas_context = canvas.getContext("2d")
    if (canvas_context == null) {
        throw new Error('canvas context is undefined')
    }

    const view = new View(canvas_context)
    const controller = new Controller(canvas)

    // TODO: UI CPU/GPU switch
        // TODO: wasm: Animator - shared by CPU/GPU
    const USE_GPU = navigator.gpu
    let model: Model
    if (USE_GPU) {
        model = await GpuModel.create(view, controller, canvas_context)
    } else {
        model = await CpuModel.create(view, controller, canvas_context)
    }
    controller.set_model(model)
    controller.activate_controls()

    // TODO: switch continuous rendering on/off
    await start_render_loop(model)
}
main()