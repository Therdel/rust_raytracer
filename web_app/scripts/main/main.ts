import init, {wasm_main, wasm_log_init, GpuRenderer} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model, GpuModel, CpuModel} from "./model.js";
import {AssetStore} from "./asset_store.js";

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

    const SCENE_BASE_PATH = "res/scenes";
    const MESH_BASE_PATH = "../../res/models";
    const asset_store = new AssetStore(SCENE_BASE_PATH, MESH_BASE_PATH)

    const scene_file_name = "gpu_test.json"
    // const scene_file_name = "cornell_box.json"
    // const scene_file_name = "infinity_santa.json"
    await asset_store.prefetch_scene_meshes(scene_file_name)

    const scene_buffer = asset_store.get_scene(scene_file_name)

    // const model = new CpuModel(view, controller, canvas_context)
    const gpu_renderer = await GpuRenderer.new(canvas.width, canvas.height, scene_buffer, asset_store)
    const model = new GpuModel(view, controller, canvas_context, gpu_renderer)
    controller.set_model(model)
}
main()