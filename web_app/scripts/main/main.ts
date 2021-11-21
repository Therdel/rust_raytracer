import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model} from "./model.js";

async function main() {
    console.log(`Main started`)
    const canvas =
        document.getElementById('screen') as HTMLCanvasElement

    const view = new View(canvas)
    const controller = new Controller(canvas)
    const model = new Model(view, controller, canvas)
    controller.set_model(model)
}
main()