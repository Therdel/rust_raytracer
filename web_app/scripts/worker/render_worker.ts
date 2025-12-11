import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"
import init, {Renderer, wasm_main} from "../../pkg/web_app.js"

class RenderWorker {
    private index: number
    private amount_workers: number
    private offscreen_canvas: OffscreenCanvas
    private canvas_context: OffscreenCanvasRenderingContext2D
    private image_data: ImageData
    private canvas_buffer_u8: Uint8Array
    private renderer: Renderer
    private dirty: boolean

    private static instance: RenderWorker

    private constructor(index: number,
                        amount_workers: number,
                        offscreen_canvas: OffscreenCanvas,
                        resize: MessageToWorker.Resize) {
        this.index = index
        this.amount_workers = amount_workers
        this.offscreen_canvas = offscreen_canvas
        // TODO: deduplicate. hint: we can't just call this.resize as this.renderer doesn't exist then
        this.offscreen_canvas.width = Math.floor(resize.width * resize.device_pixel_ratio * resize.resolution_multiplier);
        this.offscreen_canvas.height = Math.floor(resize.height * resize.device_pixel_ratio * resize.resolution_multiplier);
        this.canvas_context = this.offscreen_canvas.getContext('2d')
        this.canvas_context.resetTransform()
        this.canvas_context.scale(resize.device_pixel_ratio * resize.resolution_multiplier, resize.device_pixel_ratio * resize.resolution_multiplier)
        this.image_data = this.canvas_context.createImageData(this.offscreen_canvas.width,
                                                              this.offscreen_canvas.height)                                                      
        this.canvas_buffer_u8 = new Uint8Array(this.image_data.data.buffer)

        this.canvas_context.fillText(`worker#${index}`, 0, (index/amount_workers)*this.offscreen_canvas.height)
    
        this.renderer = new Renderer(this.image_data.width, this.image_data.height)
        this.dirty = true
    }

    static get_instance() {
        return RenderWorker.instance
    }

    static async init(message: MessageToWorker.Init) {
        await init_wasm()

        const { index, amount_workers, offscreen_canvas, resize, set_scene } = message;

        RenderWorker.instance = new RenderWorker(index,
                	                             amount_workers,
                                                 offscreen_canvas,
                                                 resize)
        await this.get_instance().set_scene(set_scene)

        // kick off rendering
        requestAnimationFrame(async (frame_time_delta) => {
            this.get_instance().render_loop(frame_time_delta)
        })
    }

    async set_scene({scene_file_buffer, meshes}: MessageToWorker.SetScene) {
        for (const [mesh_name, mesh_file_buffer] of meshes) {
            const mesh_file_buffer_u8 = new Uint8Array(mesh_file_buffer)
            this.renderer.load_mesh(mesh_name, mesh_file_buffer_u8)
        }

        const scene_file_buffer_u8 = new Uint8Array(scene_file_buffer)
        this.renderer.set_scene(scene_file_buffer_u8)
        this.dirty = true
    }

    resize({ width, height, device_pixel_ratio, resolution_multiplier }: MessageToWorker.Resize) {
        this.offscreen_canvas.width = Math.floor(width * device_pixel_ratio * resolution_multiplier);
        this.offscreen_canvas.height = Math.floor(height * device_pixel_ratio * resolution_multiplier);
        this.canvas_context = this.offscreen_canvas.getContext('2d')
        this.canvas_context.resetTransform()
        this.canvas_context.scale(device_pixel_ratio * resolution_multiplier, device_pixel_ratio * resolution_multiplier)
        this.image_data = this.canvas_context.createImageData(this.offscreen_canvas.width,
                                                              this.offscreen_canvas.height)                                                      
        this.canvas_buffer_u8 = new Uint8Array(this.image_data.data.buffer)

        this.renderer.resize_screen(this.image_data.width, this.image_data.height)
        this.dirty = true
    }

    turn_camera(message: MessageToWorker.TurnCamera) {
        const {
            drag_begin: {x: begin_x, y: begin_y},
            drag_end: {x: end_x, y: end_y},
            do_orbit
        } = message
        this.renderer.turn_camera(begin_x, begin_y, end_x, end_y, do_orbit)
        this.dirty = true
    }

    add_mesh(message: MessageToWorker.AddMesh) {
        const { mesh_url, mesh_file_buffer } = message
        const mesh_file_buffer_u8 = new Uint8Array(mesh_file_buffer)
        this.renderer.load_mesh(mesh_url, mesh_file_buffer_u8)
    }

    get_index() {
        return this.index
    }

    private render_loop(frame_time_delta) {
        // if (this.get_index() % 2 == 0)
        //     return
        if (this.dirty) {
            const y_offset = this.get_index()
            const row_jump = this.amount_workers

            // TODO: animation

            // TODO: sync putImageData with other threads with atomics

            const worker_render_start = performance.now()
            this.renderer.render_interlaced(this.canvas_buffer_u8, y_offset, row_jump)
            this.canvas_context.putImageData(this.image_data, 0, 0)
            //     TODO: thus, decouple rendering from controls.
            //           --> Renderer could periodically pull UI controls state from main, having it 100% async
            //           --> after each frame, since frames take long? Should result in natural batching
            //           --> could hurt responsiveness, tho. When we turn the camera, we want that to happen instantly.
            //               We just don't want to send more than 1 turn_camera event per frame, thus blocking worker msg queues        
            const worker_render_stop = performance.now() - worker_render_start
            console.debug(`Worker:${this.get_index()}\tResponding - Render time: ${worker_render_stop.toFixed(0)} ms`);
        }
        this.dirty = false

        requestAnimationFrame((frame_time_delta) => {
            this.render_loop(frame_time_delta)
        })
    }
}

async function init_wasm() {
    // Load wasm file, run its entry point
    await init();
    wasm_main();
}

onmessage = async ({ data: message }: MessageEvent<MessageToWorker.Message>) => {
    console.debug(`Worker:\tReceived '${message.type}'`);

    if (message.type === "MessageToWorker_Init") {
        const worker_init_start = performance.now()
        await RenderWorker.init(message)
        const worker_init_duration =
            (performance.now() - worker_init_start).toFixed(0)
    
        console.debug(`Worker:\tinit took ${worker_init_duration}ms`)
    } else if (message.type === "MessageToWorker_SetScene") {
        await RenderWorker.get_instance().set_scene(message)
    } else if (message.type === "MessageToWorker_Resize") {
        RenderWorker.get_instance().resize(message)
    } else if (message.type === "MessageToWorker_TurnCamera") {
        RenderWorker.get_instance().turn_camera(message)
    } else if (message.type === "MessageToWorker_AddMesh") {
        RenderWorker.get_instance().add_mesh(message)
    }

    const response =
        new MessageFromWorker.RenderResponse(RenderWorker.get_instance().get_index())
    postMessage(response)
}

const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds))
}

async function init_worker() {
    console.log(`Worker:\tstarted`)

    const init_message = new MessageFromWorker.Init()
    postMessage(init_message)
}
init_worker()