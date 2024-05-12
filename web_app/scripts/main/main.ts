import init, {wasm_main} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model} from "./model.js";

async function main() {
    console.log(`Main:\tstarted`)

    // Load wasm file, run its entry point
    await init();
    wasm_main();

    const canvas = document.getElementById('screen') as HTMLCanvasElement

    const view = new View(canvas)
    const controller = new Controller(canvas)
    const model = await Model.create(view, controller, canvas)
    controller.set_model(model)
}
main()