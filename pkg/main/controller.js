export class Controller {
    constructor(canvas) {
        this.canvas_resizer = document.getElementById('canvas-resizer');
        this.canvas = canvas;
        this.label_time = document.getElementById('time-measurement');
        this.label_thread_count = document.getElementById('thread-count');
        this.select = document.getElementById("select_scenes");
        this.canvas.width = this.canvas_resizer.clientWidth;
        this.canvas.height = this.canvas_resizer.clientHeight;
        this.init_listeners();
        this.deactivate_controls();
    }
    init_listeners() {
        this.canvas_resizer_observer_context = {
            call_count: 0,
            timeout_id: null,
            prev_width: this.canvas_resizer.clientWidth
        };
        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const observer = new ResizeObserver(() => this.on_canvas_resize());
        observer.observe(this.canvas_resizer);
        this.canvas.onmousedown = e => {
            const inverted_y = this.canvas.height - e.offsetY;
            this.last_mouse_down = { x: e.offsetX, y: inverted_y };
            console.log(`mouse down `, this.last_mouse_down);
        };
        this.canvas.onmouseup = e => {
            const inverted_y = this.canvas.height - e.offsetY;
            const last_mouse_up = { x: e.offsetX, y: inverted_y };
            console.log(`mouse up `, last_mouse_up);
            this.model.turn_camera(this.last_mouse_down, last_mouse_up);
        };
        // TODO: put connect canvas mouse / touch listener
        this.select.onchange = (event) => this.on_scene_select(event);
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
