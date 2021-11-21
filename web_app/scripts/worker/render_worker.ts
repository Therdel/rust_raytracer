/// <reference path="../message_to_worker.ts" />
/// <reference path="../message_from_worker.ts" />
/// <reference types="../../pkg/web_app" />

importScripts("../../pkg/web_app.js")
importScripts("../message_to_worker.js")
importScripts("../message_from_worker.js")

const SCENE_BASE_PATH = "../../../res/scenes";
const CHEAT_MODEL_PATH = "../../../res/models/santa.obj";

class RenderWorker {
    private index: number
    private amount_workers: number
    private width: number
    private height: number
    private renderer: wasm_bindgen.Renderer

    private static instance: RenderWorker
    private static cheat_obj_file: Uint8Array

    private constructor(index: number,
                amount_workers: number,
                scene: Uint8Array,
                width: number,
                height: number) {
        this.index = index;
        this.amount_workers = amount_workers;
        this.width = width
        this.height = height
        this.renderer = new wasm_bindgen.Renderer(width,
                                                  height,
                                                  scene,
                                                  RenderWorker.cheat_obj_file);
    }

    private static getInstance() {
        return RenderWorker.instance
    }

    static async init_cheat_obj() {
        this.cheat_obj_file = await fetch_into_array(CHEAT_MODEL_PATH)
    }

    static async init({ index, amount_workers, scene_file: scene_file, width, height }) {
        const scene_url = SCENE_BASE_PATH + '/' + scene_file
        const scene = await fetch_into_array(scene_url)
        RenderWorker.instance = new RenderWorker(index,
                                                 amount_workers,
                                                 scene,
                                                 width,
                                                 height)
    }

    static async scene_select({ scene_file: scene_file }) {
        const instance = RenderWorker.getInstance()
        const scene_url = SCENE_BASE_PATH + '/' + scene_file
        const scene = await fetch_into_array(scene_url)
        instance.renderer = new wasm_bindgen.Renderer(instance.width,
                                                      instance.height,
                                                      scene,
                                                      this.cheat_obj_file)
    }

    static resize({ width, height }) {
        const instance = RenderWorker.getInstance()
        instance.width = width
        instance.height = height
        instance.renderer.resize_screen(width, height)
    }

    static turn_camera(message) {
        const {
            drag_begin: { x: begin_x, y: begin_y},
            drag_end: { x: end_x, y: end_y}
        } = message
        RenderWorker.instance.renderer.turn_camera(begin_x, begin_y, end_x, end_y)
    }

    static render(buffer) {
        const instance = RenderWorker.getInstance()
        const y_offset = instance.index
        const row_jump = instance.amount_workers
        instance.renderer.render_interlaced(new Uint8Array(buffer),
                                            y_offset, row_jump)
    }

    static index() {
        return RenderWorker.instance.index
    }
}

async function init_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    await wasm_bindgen('../../pkg/web_app_bg.wasm');

    // Run main WASM entry point
    wasm_bindgen.main();
}

async function fetch_into_array(path) {
    let array_buffer = await (await fetch(path)).arrayBuffer();
    return new Uint8Array(array_buffer);
}

const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
}

async function init_worker() {
    console.log(`RenderWorker started`)

    const worker_init_start = performance.now()

    await init_wasm()
    // FIXME: These fetches are not done by the workers in parallel.
    //        Move to main?
    await RenderWorker.init_cheat_obj()

    onmessage = async ({ data }: MessageEvent<MessageToWorker_MessageWithBuffer>) => {
        const { buffer, message } = data
        console.debug(`Worker: Received '${message.type}'`);

        if (message.type === "MessageToWorker_Init") {
            await RenderWorker.init(message)
        } else if (message.type === "MessageToWorker_SceneSelect") {
            await RenderWorker.scene_select(message)
        } else if (message.type === "MessageToWorker_Resize") {
            RenderWorker.resize(message)
        } else if (message.type === "MessageToWorker_TurnCamera") {
            RenderWorker.turn_camera(message)
        }

        RenderWorker.render(buffer)

        console.debug(`Worker: Responding`);
        const response =
            new MessageFromWorker_RenderResponse(RenderWorker.index(), buffer)
        postMessage(response, [buffer])
    }
    const init_message = new MessageFromWorker_Init()
    postMessage(init_message)

    const worker_init_duration =
        (performance.now() - worker_init_start).toFixed(0)

    console.debug(`RenderWorker init took ${worker_init_duration}ms`)
}
init_worker()