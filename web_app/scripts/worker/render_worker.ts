import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"
import init, {Renderer, wasm_main} from "../../pkg/web_app.js"

const SCENE_BASE_PATH = "../../res/scenes";
const CHEAT_MODEL_PATH = "../../res/models/santa.obj";

class RenderWorker {
    private index: number
    private canvas_buffer: SharedArrayBuffer
    private canvas_buffer_u8: Uint8Array
    private amount_workers: number
    private width: number
    private height: number
    private renderer: Renderer

    private static instance: RenderWorker
    private static cheat_obj_file: Uint8Array

    private constructor(index: number,
                        buffer: SharedArrayBuffer,
                        amount_workers: number,
                        scene: Uint8Array,
                        width: number,
                        height: number) {
        this.index = index
        this.canvas_buffer = buffer
        this.canvas_buffer_u8 = new Uint8Array(this.canvas_buffer)
        this.amount_workers = amount_workers
        this.width = width
        this.height = height
        this.renderer = new Renderer(width,
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

    static async init(message: MessageToWorker.Init) {
        await init_wasm()

        // FIXME: These fetches are not done by the workers in parallel.
        //        Move to main?
        await RenderWorker.init_cheat_obj()

        const { index, buffer, amount_workers, scene_file: scene_file, width, height } = message;
        const scene_url = SCENE_BASE_PATH + '/' + scene_file
        const scene = await fetch_into_array(scene_url)

        RenderWorker.instance = new RenderWorker(index,
                                                 buffer,
                                                 amount_workers,
                                                 scene,
                                                 width,
                                                 height)
    }

    static async scene_select({ scene_file: scene_file }: MessageToWorker.SceneSelect) {
        const instance = RenderWorker.getInstance()
        const scene_url = SCENE_BASE_PATH + '/' + scene_file
        const scene = await fetch_into_array(scene_url)
        instance.renderer = new Renderer(instance.width,
                                         instance.height,
                                         scene,
                                         this.cheat_obj_file)
    }

    static resize({ width, height, buffer }: MessageToWorker.Resize) {
        const instance = RenderWorker.getInstance()
        instance.width = width
        instance.height = height
        instance.canvas_buffer = buffer
        instance.canvas_buffer_u8 = new Uint8Array(instance.canvas_buffer)
        instance.renderer.resize_screen(width, height)
    }

    static turn_camera(message: MessageToWorker.TurnCamera) {
        const {
            drag_begin: {x: begin_x, y: begin_y},
            drag_end: {x: end_x, y: end_y}
        } = message
        RenderWorker.instance.renderer.turn_camera(begin_x, begin_y, end_x, end_y)
    }

    static render() {
        const instance = RenderWorker.getInstance()
        const y_offset = instance.index
        const row_jump = instance.amount_workers
        instance.renderer.render_interlaced(instance.canvas_buffer_u8, y_offset, row_jump)
    }

    static index() {
        return RenderWorker.instance.index
    }
}

async function init_wasm() {
    // Load wasm file, run its entry point
    await init();
    wasm_main();
}

async function fetch_into_array(path) {
    let array_buffer = await (await fetch(path)).arrayBuffer();
    return new Uint8Array(array_buffer);
}

async function on_message({ data: message }: MessageEvent<MessageToWorker.Message>) {
    console.debug(`Worker:\tReceived '${message.type}'`);

    if (message.type === "MessageToWorker_Init") {
        const worker_init_start = performance.now()
        await RenderWorker.init(message)
        const worker_init_duration =
            (performance.now() - worker_init_start).toFixed(0)
    
        console.debug(`Worker:\tinit took ${worker_init_duration}ms`)
    } else if (message.type === "MessageToWorker_SceneSelect") {
        await RenderWorker.scene_select(message)
    } else if (message.type === "MessageToWorker_Resize") {
        RenderWorker.resize(message)
    } else if (message.type === "MessageToWorker_TurnCamera") {
        RenderWorker.turn_camera(message)
    }

    const worker_render_start = performance.now()
    RenderWorker.render()
    const worker_render_stop = performance.now() - worker_render_start

    console.debug(`Worker:${RenderWorker.index()}\tResponding - Render time: ${worker_render_stop.toFixed(0)} ms`);
    const response =
        new MessageFromWorker.RenderResponse(RenderWorker.index())
    postMessage(response)
}
onmessage = on_message

const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
}

async function init_worker() {
    console.log(`Worker:\tstarted`)

    const init_message = new MessageFromWorker.Init()
    postMessage(init_message)
}
init_worker()