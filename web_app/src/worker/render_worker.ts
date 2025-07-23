import { AssetStore } from "../messages/asset_store"
import * as MessageToWorker from "../messages/message_to_worker"
import * as MessageFromWorker from "../messages/message_from_worker"
import wasm_bindgen_init, {Renderer, wasm_log_init, wasm_main} from "../../wasm/pkg/wasm"

class RenderWorker {
    public static readonly index: number = RenderWorker.parse_worker_index()
    private amount_workers: number
    private canvas_buffer: SharedArrayBuffer
    private canvas_buffer_u8: Uint8Array
    private width: number
    private height: number
    private renderer: Renderer

    private static instance: RenderWorker

    private constructor(amount_workers: number,
                        canvas_buffer: SharedArrayBuffer,
                        width: number,
                        height: number) {
        this.amount_workers = amount_workers
        this.canvas_buffer = canvas_buffer
        this.canvas_buffer_u8 = new Uint8Array(this.canvas_buffer)
        this.width = width
        this.height = height
        this.renderer = new Renderer(width, height)
    }

    private static parse_worker_index() {
        if (!self.name) {
            throw new Error("Worker: name is not set")
        }
        return parseInt(self.name)
    }

    private static getInstance() {
        return RenderWorker.instance
    }

    static async init(amount_workers: number, canvas_buffer: SharedArrayBuffer, width: number, height: number) {
        RenderWorker.instance = new RenderWorker(amount_workers,
                                                 canvas_buffer,
                                                 width,
                                                 height)
    }

    static set_scene(scene_url_or_filename: string, assets_serialized: Map<string, SharedArrayBuffer>) {
        const instance = RenderWorker.getInstance()
        const asset_store = AssetStore.fromMap(assets_serialized)
        instance.renderer.set_scene(asset_store, scene_url_or_filename)
    }

    static resize(width: number, height: number, buffer: SharedArrayBuffer) {
        const instance = RenderWorker.getInstance()
        instance.width = width
        instance.height = height
        instance.canvas_buffer = buffer
        instance.canvas_buffer_u8 = new Uint8Array(instance.canvas_buffer)
        instance.renderer.resize_screen(width, height)
    }

    static turn_camera(begin: { x: number; y: number },
                       end: { x: number; y: number }) {
        const instance = RenderWorker.getInstance()
        instance.renderer.turn_camera(begin.x, begin.y, end.x, end.y)
    }

    static render() {
        const instance = RenderWorker.getInstance()
        const y_offset = RenderWorker.index
        const row_jump = instance.amount_workers
        instance.renderer.render_interlaced(instance.canvas_buffer_u8, y_offset, row_jump)
    }
}

async function on_message({ data: message }: MessageEvent<MessageToWorker.Message>) {
    const payload = message.payload
    console.debug(`Worker: received '${payload.type}'`);

    if (payload.type === "MessageToWorker_Init") {
        const worker_init_start = performance.now()
        const { amount_workers, resize } = payload
        await RenderWorker.init(amount_workers, resize.buffer, resize.width, resize.height)
        const worker_init_duration =
            (performance.now() - worker_init_start).toFixed(0)

        console.debug(`Worker: init took ${worker_init_duration}ms`)
    } else if (payload.type === "MessageToWorker_SetScene") {
        await RenderWorker.set_scene(payload.scene_url_or_filename, payload.assets_serialized)
    } else if (payload.type === "MessageToWorker_Resize") {
        RenderWorker.resize(payload.width, payload.height, payload.buffer)
    } else if (payload.type === "MessageToWorker_TurnCamera") {
        RenderWorker.turn_camera(payload.drag_begin, payload.drag_end)
    } else if (payload.type === "MessageToWorker_Render") {
        const worker_render_start = performance.now()
        RenderWorker.render()
        const worker_render_stop = performance.now() - worker_render_start
        console.debug(`Worker: render time: ${worker_render_stop.toFixed(0)} ms`);
    }

    const response = new MessageFromWorker.Message(
        message.sequence,
        RenderWorker.index)
    postMessage(response)
}

async function start_worker() {
    onmessage = on_message
    
    // Load wasm file, run its entry point
    await wasm_bindgen_init();
    wasm_main();
    wasm_log_init();
    console.log(`Worker: WASM initialized`)

    const STARTUP_SEQUENCE = 0
    postMessage(new MessageFromWorker.Message(STARTUP_SEQUENCE, RenderWorker.index))
    console.log(`Worker: started`)
}
await start_worker()