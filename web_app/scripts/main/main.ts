import init, {wasm_main} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model} from "./model.js";

async function main() {
    console.log(`Main:\tstarted`)

    // Load wasm file, run its entry point
    await init();
    wasm_main();

    const amount_workers = navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4
    const view = new View(amount_workers)
    
    // const canvas_container = document.getElementById('canvas-container') as HTMLDivElement
    // const offscreen_canvases = view.get_offscreen_canvases_once()
    // for (var i=0; i<amount_workers; ++i) {
    //     const offscreen_canvas = offscreen_canvases[i]
    //     const context_2d = offscreen_canvas.getContext('2d')
    //     context_2d.fillText(`worker#${i}`, 0, (i/amount_workers)*canvas_container.clientHeight)
    // }
    // canvas_container.onpointermove = async (event) => {
    //     console.warn(`Pointer: ${event.clientX}, ${event.clientY}`);
    // }

    const controller = new Controller()
    const model = await Model.create(amount_workers, view, controller)
    controller.set_model(model)
}
main()