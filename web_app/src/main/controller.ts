import {Model, DidHandleMessage} from "./model";

export class Controller {
    private model?: Model

    private canvas_resizer: HTMLDivElement
    private canvas_resizer_observer_context: { call_count: number, timeout_id?: number, prev_width: number }
    private canvas: HTMLCanvasElement
    private select: HTMLSelectElement

    private is_moving_camera: boolean
    private turn_camera_start_point?: { x: number, y: number }

    constructor(canvas: HTMLCanvasElement) {
        this.canvas_resizer = document.getElementById('canvas-resizer') as HTMLDivElement
        this.canvas = canvas
        this.canvas.width = this.canvas_resizer.clientWidth
        this.canvas.height = this.canvas_resizer.clientHeight
        this.select = document.getElementById("select_scenes") as HTMLSelectElement

        this.canvas_resizer_observer_context = {
            call_count: 0,
            timeout_id: undefined,
            prev_width: this.canvas_resizer.clientWidth
        }
        this.is_moving_camera = false
        this.turn_camera_start_point = undefined

        this.init_listeners()
        this.deactivate_controls()
    }

    private init_listeners() {
        // canvas resizing
        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const observer = new ResizeObserver(() => this.on_canvas_resize())
        observer.observe(this.canvas_resizer)

        // canvas camera panning 
        this.canvas.onpointerdown = pointer_event => this.start_turning_camera(pointer_event)
        this.canvas.onpointermove = async pointer_event => await this.turn_camera(pointer_event)
        const stop_turning_camera = () => { this.stop_turning_camera() } 
        this.canvas.onpointerup = stop_turning_camera
        this.canvas.onpointerleave = stop_turning_camera
        this.canvas.onpointerout   = stop_turning_camera
        this.canvas.onpointercancel = stop_turning_camera

        // scene selection
        this.select.onchange = async (event) => await this.on_set_scene(event)
    }

    // TODO: lock mouse: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_Lock_API
    private start_turning_camera(pointer_event: PointerEvent) {
        // allow camera panning when moving outside of canvas
        this.canvas.setPointerCapture(pointer_event.pointerId)

        const inverted_y = this.canvas.height - pointer_event.offsetY
        this.turn_camera_start_point = { x: pointer_event.offsetX, y: inverted_y }
        this.is_moving_camera = true
        console.debug(`pointer down `, this.turn_camera_start_point)
    }

    private async turn_camera(pointer_event: PointerEvent) {
        if (this.is_moving_camera) {
            const inverted_y = this.canvas.height - pointer_event.offsetY
            const turn_camera_end_point = { x: pointer_event.offsetX, y: inverted_y }
            console.debug(`camera move by pointer`)

            if (this.turn_camera_start_point == undefined) return
            if (this.model == undefined) throw Error(`Controller::turn_camera: Model undefined`)
            const turn_camera_result = await this.model.turn_camera(this.turn_camera_start_point, turn_camera_end_point)
            if (DidHandleMessage.YES == turn_camera_result) {
                this.turn_camera_start_point = turn_camera_end_point
            }
        }
    }

    private stop_turning_camera() {
        this.is_moving_camera = false
    }

    private on_canvas_resize() {
        // ditch observer init call
        if (this.canvas_resizer_observer_context.call_count++ == 0) {
            return
        }

        const do_resize = async () => {
            console.debug("Controller: New canvas size: ", this.get_current_canvas_size())
            this.canvas.width = this.canvas_resizer.clientWidth
            this.canvas.height = this.canvas_resizer.clientHeight

            if (this.model == undefined) throw Error(`Controller::on_canvas_resize: Model undefined`)
            await this.model.resize(this.canvas.width, this.canvas.height)
        }

        // debounce resize events - only react after 100ms of silence
        const debounce_timeout = 100
        clearTimeout(this.canvas_resizer_observer_context.timeout_id)
        this.canvas_resizer_observer_context.timeout_id =
            setTimeout(do_resize, debounce_timeout)
    }

    set_model(model: Model) {
        this.model = model
    }

    get_current_scene_file_name(): string {
        return this.select.value
    }

    get_current_canvas_size(): { width: number, height: number } {
        return {
            width: this.canvas.width,
            height: this.canvas.height
        }
    }

    deactivate_controls() {
        // TODO: disable canvas touch / drag listener
        this.canvas_resizer.style.resize = "none"
        this.select.disabled = true;
    }

    activate_controls() {
        // TODO: enable canvas touch / drag listener
        this.canvas_resizer.style.resize = "both"
        this.select.disabled = false;
    }

    private async on_set_scene(_: Event) {
        if (this.model == undefined) throw Error(`Controller::on_set_scene: Model undefined`)
        await this.model.set_scene(this.get_current_scene_file_name())

        console.debug(`Controller: Selected scene ${this.select.value}`)
    }
}