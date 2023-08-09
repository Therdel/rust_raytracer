import {View} from "./view.js";
import {Controller} from "./controller.js";
import {RenderWorkerPool} from "./render_worker_pool.js";
import { AssetStore } from "../messages/asset_store.js"
import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"
import {GpuRenderer} from "../../pkg/web_app.js"

export enum DidHandleMessage {
    YES,
    NO
}

export interface Model {
    set_scene(scene_name: string): Promise<DidHandleMessage>
    resize(width: number,
           height: number): Promise<DidHandleMessage>
    turn_camera(drag_begin: { x: number, y: number },
                drag_end:   { x: number, y: number }): Promise<DidHandleMessage>
}

export class CpuModel implements Model {
    public readonly view: View
    public readonly controller: Controller

    private state: AbstractState

    private readonly canvas: HTMLCanvasElement
    private readonly canvas_context: CanvasRenderingContext2D
    private image_data: ImageData

    public amount_workers: number
    private worker_image_buffers: SharedArrayBuffer[]
    public render_worker_pool: RenderWorkerPool

    private asset_store: AssetStore

    private constructor(view: View, controller: Controller, canvas: HTMLCanvasElement) {
        this.view = view
        this.controller = controller

        this.state = undefined

        this.canvas = canvas
        this.canvas_context = canvas.getContext("2d")
        this.init_image_data()

        this.amount_workers = navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4
        this.create_worker_image_buffers(this.canvas.width, this.canvas.height);

        this.render_worker_pool = undefined
        this.asset_store = new AssetStore()
    }

    static async create(view: View, controller: Controller, canvas: HTMLCanvasElement): Promise<CpuModel> {
        const model_core = new CpuModel(view, controller, canvas)

        const scene_file_name = controller.get_current_scene_file_name()

        await model_core.asset_store.put_scene_and_cache_assets(scene_file_name)
        const init_set_scene = new MessageToWorker.SetScene(scene_file_name, model_core.asset_store.serialize())
        for (const bam of model_core.asset_store.iterate()) {
            console.debug(`asset name: ${bam[0]}`)
        }
        model_core.state = new InitPingPong(model_core, init_set_scene)

        // start rendering
        const delegate = (message) => model_core.on_worker_message(message) // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker source: https://stackoverflow.com/a/20279485
        model_core.render_worker_pool = new RenderWorkerPool(delegate, model_core.amount_workers)
        return model_core
    }

    async set_scene(scene_name: string): Promise<DidHandleMessage> {
        await this.asset_store.put_scene_and_cache_assets(scene_name)
        const set_scene = new MessageToWorker.SetScene(scene_name, this.asset_store.serialize())
        return this.state.set_scene(set_scene)
    }

    async resize(width: number,
                 height: number): Promise<DidHandleMessage> {
        return await this.state.resize(width, height)
    }

    async turn_camera(drag_begin: { x: number, y: number },
                      drag_end: { x: number, y: number }): Promise<DidHandleMessage> {
        return await this.state.turn_camera(drag_begin, drag_end)
    }

    transition_state(state: AbstractState) {
        console.debug(`CpuModel:\ttransition: ${this.state.state_name()} -> ${state.state_name()}`)
        this.state = state
    }

    init_image_data() {
        const [width, height] = [this.canvas.width, this.canvas.height]
        this.image_data = this.canvas_context.createImageData(width, height)
    }

    create_worker_image_buffers(width: number, height: number) {
        this.worker_image_buffers = []
        const image_buf_size = width * height * 4
        for (let i = 0; i < this.amount_workers; ++i) {
            const image_buffer = new SharedArrayBuffer(image_buf_size);
            this.worker_image_buffers.push(image_buffer);
        }
    }

    get_worker_buffer(index: number): SharedArrayBuffer {
        return this.worker_image_buffers[index]
    }

    get_image_data() {
        return this.image_data
    }

    private on_worker_message(message: MessageFromWorker.Message) {
        this.state.on_message(message)
    }

    write_interlaced_worker_buffer_into_image_data(index: number, src: Uint8Array) {
        const dst = new Uint8Array(this.image_data.data.buffer)

        const y_offset = index
        const row_jump = this.render_worker_pool.amount_workers()
        const [width, height] = [this.canvas.width, this.canvas.height]

        const row_len_bytes = width * 4;
        for (let y = y_offset; y < height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes;
            const row_dst = dst.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
            const row_src = src.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
            row_dst.set(row_src);
        }
    }
}

export abstract class AbstractState {
    protected model: CpuModel

    constructor(model: CpuModel) {
        this.model = model
    }

    async set_scene(message: MessageToWorker.SetScene): Promise<DidHandleMessage> {
    // async set_scene(scene_name: string): Promise<DidHandleMessage> {
        console.log(`CpuModel<${this.state_name()}>: Didn't handle set_scene(${message})`)
        return DidHandleMessage.NO
    }

    async resize(width: number, height: number): Promise<DidHandleMessage> {
        console.log(`CpuModel<${this.state_name()}>: Didn't handle resize(`, {width, height}, `)`)
        return DidHandleMessage.NO
    }

    async turn_camera(drag_begin: { x: number; y: number },
                drag_end: { x: number; y: number }): Promise<DidHandleMessage> {
        console.log(`CpuModel<${this.state_name()}>: Didn't handle turn_camera(`, {drag_begin, drag_end}, `)`)
        return DidHandleMessage.NO
    }

    on_message(message: MessageFromWorker.Message): DidHandleMessage {
        const result = this.on_message_impl(message)
        if (result == DidHandleMessage.NO) {
            console.error(`CpuModel<${this.state_name()}>: Didn't handle message:`, message.constructor.name)
        }
        return result
    }

    protected on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
        return DidHandleMessage.NO
    }

    abstract state_name(): string
}

export class InitPingPong extends AbstractState {
    worker_responses: number = 0
    init_set_scene: MessageToWorker.SetScene

    constructor(model: CpuModel, init_set_scene: MessageToWorker.SetScene) {
        super(model);
        this.init_set_scene = init_set_scene
    }

    private send_init_and_start_first_render() {
        const amount_workers = this.model.amount_workers
        const canvas_size = this.model.controller.get_current_canvas_size()
        for (let index=0; index<amount_workers; ++index) {
            const canvas_buffer = this.model.get_worker_buffer(index);
            const message = new MessageToWorker.Init(index,
                                                        canvas_buffer,
                                                        amount_workers,
                                                        this.init_set_scene,
                                                        canvas_size.width,
                                                        canvas_size.height)
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

    constructor(model: CpuModel) {
        super(model)
        this.model.view.display_rendering_state()
        this.time_start = performance.now()
    }

    on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
        if (message.type == "MessageFromWorker_RenderResponse") {                
            const buffer = new Uint8Array(this.model.get_worker_buffer(message.index));
            this.model.write_interlaced_worker_buffer_into_image_data(message.index, buffer)

            this.worker_responses += 1
            if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                this.model.view.update_canvas(this.model.get_image_data())
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
    constructor(model: CpuModel) {
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

    async resize(width: number, height: number): Promise<DidHandleMessage> {
        this.model.init_image_data()
        this.model.create_worker_image_buffers(width, height)
        const amount_workers = this.model.render_worker_pool.amount_workers()
        for (let index=0; index<amount_workers; ++index) {
            const buffer = this.model.get_worker_buffer(index);
            const message = new MessageToWorker.Resize(width, height, buffer)
            this.model.render_worker_pool.post(index, message)
        }
        
        this.transition_to_rendering()
        return DidHandleMessage.YES
    }

    async set_scene(message: MessageToWorker.SetScene): Promise<DidHandleMessage> {
    // async set_scene(scene_file: string): Promise<DidHandleMessage> {
        // const message = new MessageToWorker.SetScene(scene_file, this.model.)
        this.post_all(message)
        this.transition_to_rendering()
        return DidHandleMessage.YES
    }

    async turn_camera(drag_begin: { x: number; y: number },
                drag_end: { x: number; y: number }): Promise<DidHandleMessage> {
        const message = new MessageToWorker.TurnCamera(drag_begin, drag_end)

        console.log("Posting turn_camera: ", message)
        this.post_all(message)
        this.transition_to_rendering()
        return DidHandleMessage.YES
    }

    state_name(): string {
        return this.constructor.name;
    }
}

export class GpuModel implements Model {
    public readonly view: View
    public readonly controller: Controller

    private state: GpuModelState.State

    private readonly canvas: HTMLCanvasElement
    private readonly canvas_context: CanvasRenderingContext2D
    private image_data: ImageData

    private readonly gpu_renderer: GpuRenderer

    constructor(view: View, controller: Controller, canvas: HTMLCanvasElement, gpu_renderer: GpuRenderer) {
        this.view = view
        this.controller = controller

        this.state = new GpuModelState.AcceptUserControl(this)

        this.canvas = canvas
        this.canvas_context = canvas.getContext("2d")
        this.init_image_data()

        this.gpu_renderer = gpu_renderer
        this.render()
    }

    transition_state(state: GpuModelState.State) {
        console.debug(`GpuModel:\ttransition: ${this.state.state_name()} -> ${state.state_name()}`)
        this.state = state
    }

    init_image_data() {
        const [width, height] = [this.canvas.width, this.canvas.height]
        this.image_data = this.canvas_context.createImageData(width, height)
    }

    get_image_data() {
        return this.image_data
    }

    get_gpu_renderer(): GpuRenderer {
        return this.gpu_renderer
    }
    
    async render() {
        this.controller.deactivate_controls()
        
        const canvas_u8 = new Uint8Array(this.get_image_data().data.buffer)

        const time_start = performance.now()
        await this.get_gpu_renderer().render(canvas_u8)
        const duration = performance.now() - time_start
        this.view.display_render_duration(duration)

        this.view.update_canvas(this.get_image_data())

        this.controller.activate_controls()
    }

    async set_scene(scene_name: string): Promise<DidHandleMessage> {
        return await this.state.set_scene(scene_name)
    }

    async resize(width: number,
                 height: number): Promise<DidHandleMessage> {
        return await this.state.resize(width, height)
    }

    async turn_camera(drag_begin: { x: number, y: number },
                      drag_end: { x: number, y: number }): Promise<DidHandleMessage> {
        return await this.state.turn_camera(drag_begin, drag_end)
    }
}

namespace GpuModelState {
    export interface State extends Model {
        on_message(message: MessageFromWorker.Message): DidHandleMessage
        state_name(): string
    }

    abstract class AbstractState implements State {
        protected model: GpuModel

        constructor(model: GpuModel) {
            this.model = model
        }

        async set_scene(scene_name: string): Promise<DidHandleMessage> {
            console.log(`GpuModelCore<${this.state_name()}>: Didn't handle set_scene(${scene_name})`)
            return DidHandleMessage.NO
        }

        async resize(width: number, height: number): Promise<DidHandleMessage> {
            console.log(`GpuModelCore<${this.state_name()}>: Didn't handle resize(`, {width, height}, `)`)
            return DidHandleMessage.NO
        }

        async turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number }): Promise<DidHandleMessage> {
            console.log(`GpuModelCore<${this.state_name()}>: Didn't handle turn_camera(`, {drag_begin, drag_end}, `)`)
            return DidHandleMessage.NO
        }

        on_message(message: MessageFromWorker.Message): DidHandleMessage {
            const result = this.on_message_impl(message)
            if (result == DidHandleMessage.NO) {
                console.error(`GpuModelCore<${this.state_name()}>: Didn't handle message:`, message.constructor.name)
            }
            return result
        }

        protected on_message_impl(message: MessageFromWorker.Message): DidHandleMessage {
            return DidHandleMessage.NO
        }

        abstract state_name(): string
    }

    export class AcceptUserControl extends AbstractState {
        constructor(model: GpuModel) {
            super(model);
        }

        async resize(width: number, height: number): Promise<DidHandleMessage> {
            this.model.init_image_data()
            this.model.get_gpu_renderer().resize_screen(width, height)
            await this.model.render()
            return DidHandleMessage.YES
        }

        async turn_camera(drag_begin: { x: number; y: number },
                          drag_end: { x: number; y: number }): Promise<DidHandleMessage> {
            this.model.get_gpu_renderer().turn_camera(drag_begin.x, drag_begin.y, drag_end.x, drag_end.y)
            await this.model.render()
            return DidHandleMessage.YES
        }

        state_name(): string {
            return this.constructor.name;
        }
    }
}
