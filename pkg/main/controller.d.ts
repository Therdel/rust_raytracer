import { Model } from "./model.js";
export declare class Controller {
    private model;
    private canvas_resizer;
    private canvas_resizer_observer_context;
    private canvas;
    private label_time;
    private label_thread_count;
    private select;
    private last_mouse_down;
    constructor(canvas: HTMLCanvasElement);
    private init_listeners;
    private on_canvas_resize;
    set_model(model: Model): void;
    get_current_scene_file(): string;
    get_current_canvas_size(): {
        width: number;
        height: number;
    };
    deactivate_controls(): void;
    activate_controls(): void;
    private on_scene_select;
}
