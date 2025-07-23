import { AssetStore } from "../messages/asset_store"
import { Controller } from "./controller"
import { GpuRenderer } from "../../wasm/pkg/wasm"
import { Model } from "./model"
import { View } from "./view"

export class GpuModel implements Model {
    public readonly asset_store: AssetStore
    public readonly view: View
    public readonly controller: Controller

    private readonly canvas_context: CanvasRenderingContext2D
    private image_data: ImageData

    private gpu_renderer: GpuRenderer

    constructor(view: View, controller: Controller, canvas_context: CanvasRenderingContext2D, asset_store: AssetStore, gpu_renderer: GpuRenderer) {
        this.view = view
        this.controller = controller

        this.canvas_context = canvas_context
        this.image_data = this.init_image_data()

        this.asset_store = asset_store

        this.gpu_renderer = gpu_renderer
        this.render()
    }

    static async create(view: View, controller: Controller, canvas_context: CanvasRenderingContext2D): Promise<GpuModel> {
        const asset_store = new AssetStore()
        const scene_file_name = controller.get_current_scene_file_name()
        await asset_store.putScene(scene_file_name)

        const { width, height } = controller.get_current_canvas_size()
        const gpu_renderer = await GpuRenderer.new(width, height, asset_store, scene_file_name)
        const gpu_model = new GpuModel(view, controller, canvas_context, asset_store, gpu_renderer)

        return gpu_model
    }

    init_image_data(): ImageData {
        const { width, height } = this.controller.get_current_canvas_size()
        this.image_data = this.canvas_context.createImageData(width, height)
        return this.image_data
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

    // FIXME: don't just recreate everything
    async set_scene(scene_name: string) {
        await this.asset_store.putScene(scene_name)

        const { width, height } = this.controller.get_current_canvas_size()
        const gpu_renderer = await GpuRenderer.new(width, height, this.asset_store, scene_name)
        this.gpu_renderer = gpu_renderer

        this.render()
    }

    async resize(width: number,
                 height: number) {
        this.init_image_data()
        this.get_gpu_renderer().resize_screen(width, height)
        await this.render()
    }

    async turn_camera(drag_begin: { x: number, y: number },
                      drag_end: { x: number, y: number }) {
        this.get_gpu_renderer().turn_camera(drag_begin.x, drag_begin.y, drag_end.x, drag_end.y)
        await this.render()
    }
}
