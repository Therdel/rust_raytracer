import init, {wasm_main, wasm_log_init} from "../../wasm/pkg/wasm"
import {View} from "./view";
import {Controller} from "./controller";
import {GpuModel, CpuModel} from "./model";

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
    if (USE_GPU) {
        const gpu_model = await GpuModel.create(view, controller, canvas_context)
        controller.set_model(gpu_model)
    } else {
        const cpu_model = await CpuModel.create(view, controller, canvas_context)
        controller.set_model(cpu_model)
    }
}
main()