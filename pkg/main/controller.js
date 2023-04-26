import { DidHandleMessage } from "./model.js";
export class Controller {
    constructor(canvas) {
        this.canvas_resizer = document.getElementById('canvas-resizer');
        this.canvas = canvas;
        this.canvas.width = this.canvas_resizer.clientWidth;
        this.canvas.height = this.canvas_resizer.clientHeight;
        this.label_time = document.getElementById('time-measurement');
        this.label_thread_count = document.getElementById('thread-count');
        this.select = document.getElementById("select_scenes");
        this.canvas_resizer_observer_context = {
            call_count: 0,
            timeout_id: null,
            prev_width: this.canvas_resizer.clientWidth
        };
        this.is_moving_camera = false;
        this.camera_move_start_point = null;
        this.init_listeners();
        this.deactivate_controls();
    }
    init_listeners() {
        // canvas resizing
        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const observer = new ResizeObserver(() => this.on_canvas_resize());
        observer.observe(this.canvas_resizer);
        // canvas camera panning 
        this.canvas.onpointerdown = pointer_event => this.start_moving_camera(pointer_event);
        this.canvas.onpointermove = pointer_event => this.move_camera(pointer_event);
        const stop_moving_camera = () => { this.stop_moving_camera(); };
        this.canvas.onpointerup = stop_moving_camera;
        this.canvas.onpointerleave = stop_moving_camera;
        this.canvas.onpointerout = stop_moving_camera;
        this.canvas.onpointercancel = stop_moving_camera;
        // scene selection
        this.select.onchange = (event) => this.on_scene_select(event);
    }
    // TODO: lock mouse: https://developer.mozilla.org/en-US/docs/Web/API/Pointer_Lock_API
    start_moving_camera(pointer_event) {
        // allow camera panning when moving outside of canvas
        this.canvas.setPointerCapture(pointer_event.pointerId);
        const inverted_y = this.canvas.height - pointer_event.offsetY;
        this.camera_move_start_point = { x: pointer_event.offsetX, y: inverted_y };
        this.is_moving_camera = true;
        console.debug(`pointer down `, this.camera_move_start_point);
    }
    move_camera(pointer_event) {
        if (this.is_moving_camera) {
            const inverted_y = this.canvas.height - pointer_event.offsetY;
            const camera_move_end_point = { x: pointer_event.offsetX, y: inverted_y };
            console.debug(`camera move by pointer`);
            const turn_camera_result = this.model.turn_camera(this.camera_move_start_point, camera_move_end_point);
            if (DidHandleMessage.YES == turn_camera_result) {
                this.camera_move_start_point = camera_move_end_point;
            }
        }
        else {
            console.debug(`inactive pointer move `);
        }
    }
    stop_moving_camera() {
        this.is_moving_camera = false;
    }
    on_canvas_resize() {
        // ditch observer init call
        if (this.canvas_resizer_observer_context.call_count++ == 0) {
            return;
        }
        const do_resize = () => {
            console.log("Controller: New canvas size: ", this.get_current_canvas_size());
            this.canvas.width = this.canvas_resizer.clientWidth;
            this.canvas.height = this.canvas_resizer.clientHeight;
            this.model.resize(this.canvas.width, this.canvas.height);
        };
        // debounce resize events - only react after 100ms of silence
        const debounce_timeout = 100;
        clearTimeout(this.canvas_resizer_observer_context.timeout_id);
        this.canvas_resizer_observer_context.timeout_id =
            setTimeout(do_resize, debounce_timeout);
    }
    set_model(model) {
        this.model = model;
    }
    get_current_scene_file() {
        return this.select.value;
    }
    get_current_canvas_size() {
        return {
            width: this.canvas.width,
            height: this.canvas.height
        };
    }
    deactivate_controls() {
        // TODO: disable canvas touch / drag listener
        this.canvas_resizer.style.resize = "none";
        this.select.disabled = true;
    }
    activate_controls() {
        // TODO: enable canvas touch / drag listener
        this.canvas_resizer.style.resize = "both";
        this.select.disabled = false;
    }
    on_scene_select(_) {
        this.model.scene_select(this.get_current_scene_file());
        console.debug(`Controller: Selected scene ${this.select.value}`);
    }
}
