import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"
import init, {Renderer, wasm_main} from "../../pkg/web_app.js"
import { AssetStore } from "../messages/asset_store.js"

class RenderWorker {
    private index: number
    private canvas_buffer: SharedArrayBuffer
    private canvas_buffer_u8: Uint8Array
    private amount_workers: number
    private width: number
    private height: number
    private renderer: Renderer

    private static instance: RenderWorker

    private constructor(index: number,
                        canvas_buffer: SharedArrayBuffer,
                        amount_workers: number,
                        width: number,
                        height: number) {
        this.index = index
        this.canvas_buffer = canvas_buffer
        this.canvas_buffer_u8 = new Uint8Array(this.canvas_buffer)
        this.amount_workers = amount_workers
        this.width = width
        this.height = height
        this.renderer = new Renderer(width, height)
    }

    private static getInstance() {
        return RenderWorker.instance
    }

    static async init(message: MessageToWorker.Init) {
        await init_wasm()

        const { index, canvas_buffer, amount_workers, set_scene, width, height } = message;

        RenderWorker.instance = new RenderWorker(index,
                                                 canvas_buffer,
                                                 amount_workers,
                                                 width,
                                                 height)
        await this.set_scene(set_scene)
    }

    static async set_scene({scene_url_or_filename, assets_serialized}: MessageToWorker.SetScene) {
        const instance = RenderWorker.getInstance()

        const asset_store = AssetStore.deserialize(assets_serialized)
        for (const [asset_name, asset_buffer] of asset_store.iterate()) {
            const asset_buffer_u8 = new Uint8Array(asset_buffer)
            instance.renderer.load_mesh(asset_name, asset_buffer_u8)
        }

        const scene_file_buffer = await asset_store.get_scene(scene_url_or_filename)
        if (scene_file_buffer === undefined) {
            throw new Error(`SetScene message with scene missing from asset store`)
        }
        const scene_file_buffer_u8 = new Uint8Array(scene_file_buffer)
        instance.renderer.set_scene(scene_file_buffer_u8)
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

async function on_message({ data: message }: MessageEvent<MessageToWorker.Message>) {
    console.debug(`Worker:\tReceived '${message.type}'`);

    if (message.type === "MessageToWorker_Init") {
        const worker_init_start = performance.now()
        await RenderWorker.init(message)
        const worker_init_duration =
            (performance.now() - worker_init_start).toFixed(0)
    
        console.debug(`Worker:\tinit took ${worker_init_duration}ms`)
    } else if (message.type === "MessageToWorker_SetScene") {
        await RenderWorker.set_scene(message)
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

async function init_worker() {
    console.log(`Worker:\tstarted`)

    const init_message = new MessageFromWorker.Init()
    postMessage(init_message)
}
init_worker()