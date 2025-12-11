import {Model, DidHandleMessage} from "./model.js";
import { View } from "./view.js";

export class Controller {
    private model: Model
    
    private slider_resolution_multiplier: HTMLInputElement
    private label_slider_resolution_multiplier: HTMLLabelElement
    private switch_camera_doorbit: HTMLInputElement

    private canvas_resizer: HTMLDivElement
    private canvas_container: HTMLDivElement
    private touchscreen: HTMLDivElement
    private canvas_resizer_observer_context: { call_count: number, timeout_id: number, prev_width: number }
    // private canvas: HTMLCanvasElement
    private select: HTMLSelectElement

    private is_moving_camera: boolean
    private turn_camera_start_point: { x: number, y: number }

    private device_pixel_ratio_change_listener: {
        remove_old_listener: () => void,
        // callback that captures 'this' & doesn't change, hence can be removed from a list using its identity
        on_change_listener: () => void
    }

    // add mesh button
    private user_meshes: Map<string, SharedArrayBuffer>
    private add_mesh_url_input: HTMLInputElement
    private add_mesh_file_name_input: HTMLInputElement
    private add_mesh_url_list: HTMLUListElement
    private add_mesh_button: HTMLButtonElement
    private add_mesh_user_feedback: HTMLLabelElement
    private add_mesh_file_chooser: HTMLInputElement

    constructor() {
        this.canvas_resizer = document.getElementById("canvas-resizer") as HTMLDivElement
        this.canvas_container = document.getElementById('canvas-container') as HTMLDivElement
        this.touchscreen = document.getElementById("touchscreen") as HTMLDivElement

        this.select = document.getElementById("select_scenes") as HTMLSelectElement

        this.canvas_resizer_observer_context = {
            call_count: 0,
            timeout_id: null,
            // TODO: DPI-awareness?
            prev_width: this.canvas_resizer.clientWidth
        }
        this.is_moving_camera = false
        this.turn_camera_start_point = null

        this.init_listeners()
        this.deactivate_controls()
    }

    private init_listeners() {
        // canvas resolution multiplier
        this.slider_resolution_multiplier = document.getElementById("slider_resolution_multiplier") as HTMLInputElement
        this.label_slider_resolution_multiplier = document.getElementById("label_slider_resolution_multiplier") as HTMLLabelElement
        this.slider_resolution_multiplier.oninput = () => this.on_slider_resolution_multiplier_change(false)
        this.on_slider_resolution_multiplier_change(true)

        this.switch_camera_doorbit = document.getElementById("switch-camera-doorbit") as HTMLInputElement

        // canvas resizing
        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const observer = new ResizeObserver(() => this.on_canvas_resize())
        observer.observe(this.canvas_resizer)

        // canvas camera panning
        this.touchscreen.onpointerdown = async pointer_event => this.start_turning_camera(pointer_event)
        this.touchscreen.onpointermove = async pointer_event => this.turn_camera(pointer_event)
        const stop_moving_camera = async () => { this.stop_moving_camera() } 
        this.touchscreen.onpointerup = stop_moving_camera 
        this.touchscreen.onpointerleave = stop_moving_camera
        // TODO: pointerout misfires during pointer moving inside
        this.touchscreen.onpointerout   = stop_moving_camera
        this.touchscreen.onpointercancel = stop_moving_camera

        // scene selection
        this.select.onchange = async (event) => await this.on_set_scene(event)

        // device pixel ratio changes
        this.device_pixel_ratio_change_listener = {
            remove_old_listener: null,
            on_change_listener: () => this.on_device_pixel_ratio_change()
        }
        this.device_pixel_ratio_change_listener.on_change_listener()

        // add mesh button
        this.user_meshes = new Map<string, SharedArrayBuffer>()
        this.add_mesh_url_input = document.getElementById('add_mesh_url_input') as HTMLInputElement
        this.add_mesh_file_name_input = document.getElementById('add_mesh_file_name_input') as HTMLInputElement
        this.add_mesh_url_list = document.getElementById('add_mesh_url_list') as HTMLUListElement
        this.add_mesh_button = document.getElementById('add_mesh_button') as HTMLButtonElement
        this.add_mesh_user_feedback = document.getElementById('add_mesh_user_feedback') as HTMLLabelElement
        this.add_mesh_button.onclick = async (event) => this.on_add_mesh_button(event)

        // mesh url input
        this.add_mesh_url_input.oninput = (_) => this.on_add_mesh_url_input_inputted()
        this.on_add_mesh_url_input_inputted()

        // mesh file chooser
        this.add_mesh_file_chooser = document.getElementById('add_mesh_file_chooser') as HTMLInputElement;
        this.add_mesh_file_chooser.onchange = async (_) => await this.on_add_mesh_file_choose()
    }

    public resolution_multiplier(): number {
        const resolution_multiplier: number = Number.parseFloat(this.slider_resolution_multiplier.value)
        return resolution_multiplier
    }

    private on_slider_resolution_multiplier_change(first_call: boolean) {
        const resolution_multiplier: number = this.resolution_multiplier()
        this.label_slider_resolution_multiplier.innerHTML = `Resolution x${resolution_multiplier.toFixed(2)}`
        
        if (!first_call) {
            console.debug(`Controller: New resolution multiplier: ${resolution_multiplier}`)
            const { width, height } = this.get_current_canvas_size()
            this.model.resize(width, height, window.devicePixelRatio, resolution_multiplier)
        }
    }

    // source: https://developer.mozilla.org/en-US/docs/Web/API/Window/devicePixelRatio
    private on_device_pixel_ratio_change() {
        const first_call = this.device_pixel_ratio_change_listener.remove_old_listener == null
        if (!first_call) {
            this.device_pixel_ratio_change_listener.remove_old_listener();
            console.debug(`Controller: New canvas device pixel ratio: ${window.devicePixelRatio}`)
            const { width, height } = this.get_current_canvas_size()
            this.model.resize(width, height, window.devicePixelRatio, this.resolution_multiplier())
        }

        const mqString = `(resolution: ${window.devicePixelRatio}dppx)`;
        const media = matchMedia(mqString);
        media.addEventListener("change", this.device_pixel_ratio_change_listener.on_change_listener);
        this.device_pixel_ratio_change_listener.remove_old_listener = () => {
            media.removeEventListener("change", this.device_pixel_ratio_change_listener.on_change_listener);
        };
    }

    // TODO: lock mouse: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_Lock_API
    private start_turning_camera(pointer_event: PointerEvent) {
        // allow camera panning when moving outside of canvas
        this.touchscreen.setPointerCapture(pointer_event.pointerId)

        // FIXME: There's a bug that after device zoom, the resize is stopped after the first frame.
        //        Looks like it mistakes it with highlighting something
        const pixel_ratio = window.devicePixelRatio
        const resolution_multiplier = this.resolution_multiplier()
        const inverted_y = this.canvas_resizer.clientHeight * pixel_ratio * resolution_multiplier - pointer_event.offsetY * pixel_ratio * resolution_multiplier
        this.turn_camera_start_point = { x: pointer_event.offsetX * pixel_ratio * resolution_multiplier, y: inverted_y }
        this.is_moving_camera = true
        console.debug(`pointer down `, this.turn_camera_start_point)
    }

    private turn_camera(pointer_event: PointerEvent) {
        console.debug(`turn_camera: is_moving=${this.is_moving_camera}`)
        if (this.is_moving_camera) {
            const pixel_ratio = window.devicePixelRatio
            const resolution_multiplier = this.resolution_multiplier()
            const inverted_y = this.canvas_resizer.clientHeight * pixel_ratio * resolution_multiplier - pointer_event.offsetY * pixel_ratio * resolution_multiplier
            const camera_move_end_point = { x: pointer_event.offsetX * pixel_ratio * resolution_multiplier, y: inverted_y }
            console.debug(`camera move by pointer`)

            const do_orbit = this.switch_camera_doorbit.checked

            const turn_camera_result = this.model.turn_camera(this.turn_camera_start_point, camera_move_end_point, do_orbit)
            if (DidHandleMessage.YES == turn_camera_result) {
                this.turn_camera_start_point = camera_move_end_point
            }
        }
    }

    private stop_moving_camera() {
        this.is_moving_camera = false
    }

    private on_canvas_resize() {
        // ditch observer init call
        if (this.canvas_resizer_observer_context.call_count++ == 0) {
            return
        }

        const do_resize = () => {
            const { width, height } = this.get_current_canvas_size()
            console.debug(`Controller: New canvas size: ${width} | ${height}`)
            this.model.resize(width, height, window.devicePixelRatio, this.resolution_multiplier())
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
            width: this.canvas_resizer.clientWidth,
            height: this.canvas_resizer.clientHeight
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
        await this.model.set_scene(this.get_current_scene_file_name())
        console.debug(`Controller: Selected scene ${this.select.value}`)
    }

    private async on_add_mesh_button(_) {
        const mesh_url = this.add_mesh_url_input.value
        if (this.user_meshes.has(mesh_url)) {
            this.add_mesh_user_feedback.innerHTML = `Mesh already exists.`
            console.info(this.add_mesh_user_feedback.innerHTML)
            return
        }
        // TODO: handle faulty URL
        // TODO: fade text after some time, so it's a notification
        const array_buffer = await (await fetch(mesh_url)).arrayBuffer();
        const mesh_file_buffer = new SharedArrayBuffer(array_buffer.byteLength)
        new Uint8Array(mesh_file_buffer).set(new Uint8Array(array_buffer))
        this.user_meshes.set(mesh_url, mesh_file_buffer)
        this.add_mesh_user_feedback.innerHTML = `Mesh added.`
        console.info(this.add_mesh_user_feedback.innerHTML)

        const url_list_elem = document.createElement(`li`)
        const mesh_size_mb = (mesh_file_buffer.byteLength/1000000).toFixed(2)
        url_list_elem.innerHTML = `${mesh_url} - size=${mesh_size_mb}MB`
        this.add_mesh_url_list.appendChild(url_list_elem)

        // TODO: Deactivate the interface instead of having to handle this result here
        this.model.add_mesh(mesh_url, mesh_file_buffer)
    }

    private on_add_mesh_url_input_inputted() {
        const input = this.add_mesh_url_input.value
        // source: https://stackoverflow.com/a/43467144
        function isValidHttpUrl(string: string): boolean {
            let url: URL
            try {
                url = new URL(string)
            } catch (_) {
                return false;  
            }
            return url.protocol === "http:" || url.protocol === "https:";
        }
        if (isValidHttpUrl(input)) {
            // source: https://stackoverflow.com/a/36756650
            const file_name = input.split('#')[0].split('?')[0].split('/').pop();
            this.add_mesh_file_name_input.value = file_name
        } else {
            this.add_mesh_file_name_input.value = ""
        }
    }

    private async on_add_mesh_file_choose() {
        const files = this.add_mesh_file_chooser.files
        for (var i=0; i < files.length; ++i) {
            const file = files.item(i)
            const file_array_buffer = await file.arrayBuffer()
            const file_shared_array_buffer = new SharedArrayBuffer(file.size)
            new Uint8Array(file_shared_array_buffer).set(new Uint8Array(file_array_buffer))

            // const file_u8 = new Uint8Array(file_shared_array_buffer)
            const file_str = new TextDecoder().decode(file_array_buffer)
            console.error(`Loaded Mesh name=${file.name} size=${file_shared_array_buffer.byteLength}, content=${file_str}`)
        }
    }
}