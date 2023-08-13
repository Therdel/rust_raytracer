import init, {wasm_main, wasm_log_init, GpuRenderer} from "../../pkg/web_app.js"
import {View} from "./view.js";
import {Controller} from "./controller.js";
import {Model, GpuModel, CpuModel} from "./model.js";

class MeshStore {
    mesh_base_url: string
    mesh_cache: Map<string, Uint8Array>

    constructor(mesh_base_url: string) {
        this.mesh_base_url = mesh_base_url;
        this.mesh_cache = new Map()
    }

    async load_mesh(name: string) {
        const mesh_url = this.mesh_base_url + '/' + name
        const mesh = await fetch_into_array(mesh_url)
        console.log(`Loaded mesh '${name}'  size ${mesh.length}`)
        this.mesh_cache.set(name, mesh)
    }

    get_mesh(name: string): Uint8Array {
        const mesh = this.mesh_cache.get(name)
        if (mesh == undefined) {
            throw new Error(`mesh ${name} is undefined`)
        }
        console.log(`get_mesh(${name}): size=${mesh.length}`)
        return mesh
    }
}
    
async function fetch_into_array(path: string): Promise<Uint8Array> {
    const buffer = await (await fetch(path)).arrayBuffer()
    return new Uint8Array(buffer)
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

    const SCENE_BASE_PATH = "res/scenes";
    const scene_url = SCENE_BASE_PATH + '/' + "gpu_test.json"
    // const scene_url = SCENE_BASE_PATH + '/' + "cornell_box.json"
    // const scene_url = SCENE_BASE_PATH + '/' + "infinity_santa.json"
    var scene_buffer = await fetch_into_array(scene_url)
    
    const MESH_BASE_PATH = "../../res/models";
    const mesh_store = new MeshStore(MESH_BASE_PATH)
    var scene_string = new TextDecoder().decode(scene_buffer)
    const scene_js = JSON.parse(scene_string)
    const meshes = scene_js.meshes
    if (meshes != undefined) {
        for (const mesh of meshes) {
            await mesh_store.load_mesh(mesh.file_name)
        }
    }

    // const model = new CpuModel(view, controller, canvas, canvas_context)
    const gpu_renderer = await GpuRenderer.new(canvas.width, canvas.height, scene_buffer, mesh_store)
    const model = new GpuModel(view, controller, canvas, canvas_context, gpu_renderer)
    controller.set_model(model)
}
main()