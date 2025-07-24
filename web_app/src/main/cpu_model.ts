import { AssetStore } from "../messages/asset_store"
import { Controller } from "./controller"
import * as MessageToWorker from "../messages/message_to_worker"
import { Model } from "./model"
import { RenderWorkerPool } from "./render_worker_pool"
import { View } from "./view"

export class CpuModel implements Model {
    public readonly view: View
    public readonly controller: Controller

    private asset_store: AssetStore
    public render_worker_pool: RenderWorkerPool

    private readonly canvas_context: CanvasRenderingContext2D
    private image_data: ImageData

    // TODO: Either one SharedArrayBuffer or many regular buffers to avoid CORS problems
    private worker_image_buffers: SharedArrayBuffer[]

    private constructor(view: View, controller: Controller, canvas_context: CanvasRenderingContext2D, asset_store: AssetStore, render_worker_pool: RenderWorkerPool) {
        this.view = view
        this.controller = controller

        this.asset_store = asset_store
        this.render_worker_pool = render_worker_pool

        this.canvas_context = canvas_context
        this.image_data = this.init_image_data()

        const { width, height } = this.controller.get_current_canvas_size()
        // TODO: rename to worker_render_buffers
        this.worker_image_buffers = this.create_worker_image_buffers(width, height)
    }

    static async create(view: View, controller: Controller, canvas_context: CanvasRenderingContext2D): Promise<CpuModel> {
        const asset_store = new AssetStore()

        const amount_workers = navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4
        const render_worker_pool = await RenderWorkerPool.create(amount_workers)

        const model = new CpuModel(view, controller, canvas_context, asset_store, render_worker_pool)
        const init = await model.init_workers()
        const scene_file_name = controller.get_current_scene_file_name()
        const setScene = await model.set_scene(scene_file_name)

        return model
    }

    private async init_workers() {
        const { width, height } = this.controller.get_current_canvas_size()

        const mapFn = (workerIndex: number): MessageToWorker.Payload => {
            const canvas_buffer = this.get_worker_buffer(workerIndex)
            const resize_message = new MessageToWorker.Resize(width, height, canvas_buffer)
            const init_message = new MessageToWorker.Init(this.render_worker_pool.amountWorkers(), resize_message)

            return init_message
        }

        return this.render_worker_pool.dispatch(mapFn)
    }

    async render(){
        // this.view.display_rendering_state()
        const time_start = performance.now()

        const message = new MessageToWorker.Render()
        const mapFn = (workerIndex: number) => message
        const reduceFn = (workerIndex: number) => {
            const buffer = new Uint8Array(this.get_worker_buffer(workerIndex))
            this.write_interlaced_worker_buffer_into_image_data(workerIndex, buffer)
        }
        await this.render_worker_pool.dispatch(mapFn, reduceFn)

        const duration = performance.now() - time_start
        this.view.display_render_duration(duration)

        // TODO: maybe do update_canvas in the render loop?
        // TODO: model updates canvas, canvas calls model.get_current_frame() 
        //       would mean that we keep a duplicate, clean frame-buffer here that the view can get, or we just push it only when it's consistent
        this.view.update_canvas(this.get_image_data())
        
        console.debug(`Render finished`)
    }

    async set_scene(scene_name: string) {
        await this.asset_store.putScene(scene_name)
        const assets_serialized = this.asset_store.getAssetsMap()

        const message = new MessageToWorker.SetScene(scene_name, assets_serialized)
        await this.render_worker_pool.dispatch(() => message)

        console.debug(`Set scene finished`)
    }

    async resize(width: number,
                 height: number) {        
        // TODO: invalidate current frame
        this.init_image_data()
        this.create_worker_image_buffers(width, height)

        const mapFn = (workerIndex: number): MessageToWorker.Payload => {
            const buffer = this.get_worker_buffer(workerIndex)
            return new MessageToWorker.Resize(width, height, buffer)
        }
        await this.render_worker_pool.dispatch(mapFn)

        console.debug(`Resize finished`)
    }

    async turn_camera(drag_begin: { x: number, y: number },
                      drag_end: { x: number, y: number }) {
        const message = new MessageToWorker.TurnCamera(drag_begin, drag_end)

        console.log("Posting turn_camera: ", message)
        await this.render_worker_pool.dispatch(() => message)

        console.debug(`Turn camera finished`)
    }

    private init_image_data() {
        const { width, height } = this.controller.get_current_canvas_size()
        this.image_data = this.canvas_context.createImageData(width, height)
        return this.image_data
    }

    private create_worker_image_buffers(width: number, height: number): SharedArrayBuffer[] {
        this.worker_image_buffers = []
        const image_buf_size = width * height * 4
        this.worker_image_buffers = Array.from({ length: this.render_worker_pool.amountWorkers() },
            () => new SharedArrayBuffer(image_buf_size))
        return this.worker_image_buffers
    }

    private get_worker_buffer(index: number): SharedArrayBuffer {
        return this.worker_image_buffers[index]
    }

    private get_image_data() {
        return this.image_data
    }

    private write_interlaced_worker_buffer_into_image_data(index: number, src: Uint8Array) {
        const dst = new Uint8Array(this.image_data.data.buffer)

        const y_offset = index
        const row_jump = this.render_worker_pool.amountWorkers()
        const { width, height } = this.controller.get_current_canvas_size()

        const row_len_bytes = width * 4
        for (let y = y_offset; y < height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes
            const row_dst = dst.subarray(row_begin_offset, row_begin_offset + row_len_bytes)
            const row_src = src.subarray(row_begin_offset, row_begin_offset + row_len_bytes)
            row_dst.set(row_src)
        }
    }
}