import {View} from "./view.js";
import {Controller} from "./controller.js";
import {RenderWorkerPool} from "./render_worker_pool.js";
import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"

export enum DidHandleMessage {
    YES,
    NO
}

export class Model {
    private readonly core: ModelCore

    private constructor(core: ModelCore) {
        this.core = core
    }

    static async create(amount_workers: number, view: View, controller: Controller): Promise<Model> {
        const model_core = await ModelCore.create(amount_workers, view, controller)
        return new Model(model_core)
    }

    async set_scene(scene_name: string): Promise<DidHandleMessage> {
        return await this.core.set_scene(scene_name)
    }

    resize(width: number,
           height: number,
           devicePixelRatio: number,
           resolution_multiplier: number): DidHandleMessage {
        return this.core.resize(width, height, devicePixelRatio, resolution_multiplier)
    }

    turn_camera(drag_begin: { x: number, y: number },
                drag_end: { x: number, y: number },
                do_orbit: boolean): DidHandleMessage {
        return this.core.turn_camera(drag_begin, drag_end, do_orbit)
    }

    add_mesh(mesh_url: string, mesh_file_buffer: SharedArrayBuffer) {
        return this.core.add_mesh(mesh_url, mesh_file_buffer)
    }
}

class ModelCore {
    public readonly view: View
    public readonly controller: Controller

    private state: ModelState.AbstractState

    // private readonly canvas: HTMLCanvasElement
    // private readonly canvas_context: CanvasRenderingContext2D
    // private image_data: ImageData

    public amount_workers: number
    // private worker_image_buffers: SharedArrayBuffer[]
    public render_worker_pool: RenderWorkerPool

    private scene: { file_name: string, file_buffer: SharedArrayBuffer }
    private mesh_cache: Map<string, SharedArrayBuffer>

    private constructor(amount_workers: number, view: View, controller: Controller) {
        this.view = view
        this.controller = controller

        this.state = undefined

        // this.canvas = canvas
        // this.canvas_context = canvas.getContext("2d")
        // this.init_image_data()

        this.amount_workers = amount_workers
        // this.create_worker_image_buffers(this.canvas.width, this.canvas.height);

        this.render_worker_pool = undefined
    }

    static async create(amount_workers: number, view: View, controller: Controller): Promise<ModelCore> {
        const model_core = new ModelCore(amount_workers, view, controller)
        
        const scene_file_name = controller.get_current_scene_file_name()
        await model_core.fetch_scene_and_cache_meshes(scene_file_name)
        const init_set_scene = new MessageToWorker.SetScene(model_core.scene.file_buffer, model_core.mesh_cache)
        const offscreen_canvases = view.get_offscreen_canvases_once()
        const { width, height } = controller.get_current_canvas_size()
        const init_resize = new MessageToWorker.Resize(width, height, window.devicePixelRatio, controller.resolution_multiplier())
        model_core.state = new ModelState.InitPingPong(model_core, offscreen_canvases, init_resize, init_set_scene)

        // start rendering
        const delegate = (message) => model_core.on_worker_message(message) // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker source: https://stackoverflow.com/a/20279485
        model_core.render_worker_pool = new RenderWorkerPool(delegate, model_core.amount_workers)
        return model_core
    }

    async set_scene(scene_name: string): Promise<DidHandleMessage> {
        await this.fetch_scene_and_cache_meshes(scene_name)
        const set_scene = new MessageToWorker.SetScene(this.scene.file_buffer, this.mesh_cache)
        return this.state.set_scene(set_scene)
    }

    resize(width: number,
           height: number,
           devicePixelRatio: number,
           resolution_multiplier: number): DidHandleMessage {
        return this.state.resize(width, height, devicePixelRatio, resolution_multiplier)
    }

    turn_camera(drag_begin: { x: number, y: number },
                drag_end: { x: number, y: number },
                do_orbit: boolean): DidHandleMessage {
        return this.state.turn_camera(drag_begin, drag_end, do_orbit)
    }

    add_mesh(mesh_url: string, mesh_obj: SharedArrayBuffer): DidHandleMessage {
        return this.state.add_mesh(mesh_url, mesh_obj)
    }

    private async fetch_scene_and_cache_meshes(scene_file_name: string) {
        const scene_file_buffer = await this.fetch_scene(scene_file_name)
        this.scene = { file_name: scene_file_name, file_buffer: scene_file_buffer }

        await this.cache_scene_meshes(scene_file_buffer)
    }

    private async fetch_scene(file_name: string): Promise<SharedArrayBuffer> {
        const SCENES_BASE_PATH = "../res/scenes";
        const url = SCENES_BASE_PATH + '/' + file_name

        let file_buffer_u8 = await this.fetch_into_array(url)
        let file_buffer_shared = new SharedArrayBuffer(file_buffer_u8.byteLength)
        new Uint8Array(file_buffer_shared).set(file_buffer_u8)

        return file_buffer_shared
    }

    // parse the scene to cache its meshes
    private async cache_scene_meshes(scene_file_buffer: SharedArrayBuffer) {
        this.mesh_cache = new Map<string, SharedArrayBuffer>()
        const MODELS_BASE_PATH = "../res/models";

        const scene_file_buffer_nonshared_for_decoding = new ArrayBuffer(scene_file_buffer.byteLength)
        const scene_file_buffer_u8 = new Uint8Array(scene_file_buffer_nonshared_for_decoding)
        scene_file_buffer_u8.set(new Uint8Array(scene_file_buffer))
        const scene_str = new TextDecoder().decode(scene_file_buffer_u8)
        const scene = JSON.parse(scene_str)

        // TODO validate scene, throw if of wrong format
        // "meshes": [
        //     {
        //         "name": "bunny",
        //         "file_name": "bunny.obj",
        //         "winding_order": "CounterClockwise",
        //         "material": "someShinyGreen"
        //     }
        // ]
        // const scene_format = {
        //     "meshes": [
        //         {
        //             "name": "bunny",
        //             "file_name": "bunny.obj",
        //             "winding_order": "CounterClockwise",
        //             "material": "someShinyGreen"
        //         }
        //     ]
        // }

        if ("meshes" in scene) {
            for (const mesh of scene.meshes) {
                const mesh_file_name: string = mesh.file_name
                if (this.mesh_cache.has(mesh_file_name)) {
                    continue
                }
                
                const mesh_url = MODELS_BASE_PATH + '/' + mesh_file_name
                let mesh_file_buffer_u8 = await this.fetch_into_array(mesh_url)
                let mesh_file_buffer_shared = new SharedArrayBuffer(mesh_file_buffer_u8.byteLength)
                new Uint8Array(mesh_file_buffer_shared).set(mesh_file_buffer_u8)
                this.mesh_cache.set(mesh_file_name, mesh_file_buffer_shared)
                console.debug(`ModelCore cached new mesh: name=${mesh_file_name}`)
            }
        }
    }

    transition_state(state: ModelState.AbstractState) {
        console.debug(`Model:\ttransition: ${this.state.state_name()} -> ${state.state_name()}`)
        this.state = state
    }

    // init_image_data() {
    //     const [width, height] = [this.canvas.width, this.canvas.height]
    //     this.image_data = this.canvas_context.createImageData(width, height)
    // }

    // create_worker_image_buffers(width: number, height: number) {
    //     this.worker_image_buffers = []
    //     const image_buf_size = width * height * 4
    //     for (let i = 0; i < this.amount_workers; ++i) {
    //         const image_buffer = new SharedArrayBuffer(image_buf_size);
    //         this.worker_image_buffers.push(image_buffer);
    //     }
    // }

    // get_worker_buffer(index: number): SharedArrayBuffer {
    //     return this.worker_image_buffers[index]
    // }

    // get_image_data() {
    //     return this.image_data
    // }

    private on_worker_message(message: MessageFromWorker.Message) {
        this.state.on_message(message)
    }

    // write_interlaced_worker_buffer_into_image_data(index: number, src: Uint8Array) {
    //     const dst = new Uint8Array(this.image_data.data.buffer)

    //     const y_offset = index
    //     const row_jump = this.render_worker_pool.amount_workers()
    //     const [width, height] = [this.canvas.width, this.canvas.height]

    //     const row_len_bytes = width * 4;
    //     for (let y = y_offset; y < height; y += row_jump) {
    //         const row_begin_offset = y * row_len_bytes;
    //         const row_dst = dst.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
    //         const row_src = src.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
    //         row_dst.set(row_src);
    //     }
    // }
        
    private async fetch_into_array(path) {
        let array_buffer = await (await fetch(path)).arrayBuffer();
        return new Uint8Array(array_buffer);
    }
}

namespace ModelState {
    export abstract class AbstractState {
        protected model: ModelCore

        constructor(model: ModelCore) {
            this.model = model
        }

        set_scene(message: MessageToWorker.SetScene): DidHandleMessage {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle set_scene(${message})`)
            return DidHandleMessage.NO
        }

        resize(width: number, height: number, devicePixelRatio: number, resolution_multiplier: number): DidHandleMessage {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle resize(`, {width, height, devicePixelRatio, resolution_multiplier}, `)`)
            return DidHandleMessage.NO
        }

        turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number },
                    do_orbit: boolean): DidHandleMessage {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle turn_camera(`, {drag_begin, drag_end, do_orbit}, `)`)
            return DidHandleMessage.NO
        }

        on_message(message: MessageFromWorker.Message): DidHandleMessage {
            const result = this.on_message_impl(message)
            if (result == DidHandleMessage.NO) {
                console.error(`ModelCore<${this.state_name()}>: Didn't handle message:`, message.constructor.name)
            }
            return result
        }

        add_mesh(mesh_url: string, mesh_obj: SharedArrayBuffer): DidHandleMessage {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle add_mesh(${mesh_url})`)
            return DidHandleMessage.NO
        }

        protected on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
            return DidHandleMessage.NO
        }

        abstract state_name(): string
    }

    export class InitPingPong extends AbstractState {
        worker_responses: number
        offscreen_canvases: OffscreenCanvas[]
        init_resize: MessageToWorker.Resize
        init_set_scene: MessageToWorker.SetScene

        constructor(model: ModelCore,
                    offscreen_canvases: OffscreenCanvas[],
                    init_resize: MessageToWorker.Resize,
                    init_set_scene: MessageToWorker.SetScene) {
            super(model)
            this.worker_responses = 0
            this.offscreen_canvases = offscreen_canvases
            this.init_resize = init_resize
            this.init_set_scene = init_set_scene
        }

        private send_init_and_start_first_render() {
            const amount_workers = this.model.amount_workers
            // const canvas_size = this.model.controller.get_current_canvas_size()
            for (let index=0; index<amount_workers; ++index) {
                // const canvas_buffer = this.model.get_worker_buffer(index)
                const offscreen_canvas = this.offscreen_canvases[index]
                const message = new MessageToWorker.Init(index,
                                                         amount_workers,
                                                         offscreen_canvas,
                                                         this.init_resize,
                                                         this.init_set_scene)
                this.model.render_worker_pool.post(index, message)
            }
            this.model.transition_state(new Rendering(this.model))
        }

        on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
            if (message.type == "MessageFromWorker_Init") {
                this.worker_responses += 1
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    this.send_init_and_start_first_render()
                }
                return DidHandleMessage.YES
            }
            return DidHandleMessage.NO
        }

        state_name(): string {
            return this.constructor.name;
        }
    }

    class Rendering extends AbstractState {
        worker_responses: number = 0
        time_start: number

        constructor(model: ModelCore) {
            super(model)
            this.model.view.display_rendering_state()
            this.time_start = performance.now()
        }

        on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
            if (message.type == "MessageFromWorker_RenderResponse") {                
                // const buffer = new Uint8Array(this.model.get_worker_buffer(message.index));
                // this.model.write_interlaced_worker_buffer_into_image_data(message.index, buffer)

                this.worker_responses += 1
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    // this.model.view.update_canvas(this.model.get_image_data())
                    this.model.transition_state(new AcceptUserControl(this.model))
                    this.display_render_time()
                }
                return DidHandleMessage.YES
            }
            return DidHandleMessage.NO
        }

        private display_render_time() {
            const duration = performance.now() - this.time_start
            this.model.view.display_render_duration(duration)
        }

        state_name(): string {
            return this.constructor.name;
        }
    }

    class AcceptUserControl extends AbstractState {
        constructor(model: ModelCore) {
            super(model);
            this.model.controller.activate_controls()
        }

        private transition_to_rendering() {
            this.model.controller.deactivate_controls()
            this.model.transition_state(new Rendering(this.model))
        }

        private post_all(message: MessageToWorker.Message) {
            const amount_workers = this.model.render_worker_pool.amount_workers()
            for (let index=0; index<amount_workers; ++index) {
                this.model.render_worker_pool.post(index, message)
            }
        }

        resize(width: number, height: number, devicePixelRatio: number, resolution_multiplier: number): DidHandleMessage {
            const amount_workers = this.model.render_worker_pool.amount_workers()
            for (let index=0; index<amount_workers; ++index) {
                const message = new MessageToWorker.Resize(width, height, devicePixelRatio, resolution_multiplier)
                this.model.render_worker_pool.post(index, message)
            }

            this.transition_to_rendering()
            return DidHandleMessage.YES
        }

        set_scene(message: MessageToWorker.SetScene): DidHandleMessage {
            this.post_all(message)
            this.transition_to_rendering()
            return DidHandleMessage.YES
        }

        turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number },
                    do_orbit: boolean): DidHandleMessage {
            const message = new MessageToWorker.TurnCamera(drag_begin, drag_end, do_orbit)

            console.log("Posting turn_camera: ", message)
            this.post_all(message)
            this.transition_to_rendering()
            return DidHandleMessage.YES
        }

        add_mesh(mesh_url: string, mesh_obj: SharedArrayBuffer): DidHandleMessage {
            const message = new MessageToWorker.AddMesh(mesh_url, mesh_obj)

            console.log("Posting add_mesh: ", message)
            this.post_all(message)
            this.transition_to_rendering()
            return DidHandleMessage.YES
        }

        state_name(): string {
            return this.constructor.name;
        }
    }
}
