import { Model } from "./model.js";
export declare class Controller {
    private model;
    private canvas_resizer;
    private canvas_resizer_observer_context;
    private canvas;
    private select;
    private is_moving_camera;
    private camera_move_start_point;
    constructor(canvas: HTMLCanvasElement);
    private init_listeners;
    private start_moving_camera;
    private move_camera;
    private stop_moving_camera;
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
