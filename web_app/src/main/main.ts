import wasm_bindgen_init, { wasm_main, wasm_log_init, InitOutput, initThreadPool } from "../../wasm/pkg/wasm"
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

async function init_wasm() {
    const output: InitOutput = await wasm_bindgen_init()

    const isSharedArrayBuffer = output.memory.buffer instanceof SharedArrayBuffer
    if (!isSharedArrayBuffer) {
        throw new Error('WebAssembly memory buffer is not a SharedArrayBuffer');
    }
    
    await initThreadPool(navigator.hardwareConcurrency)
    wasm_main();
    wasm_log_init();
}

async function main() {
    console.log(`Main:\tstarted`)

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
await main()