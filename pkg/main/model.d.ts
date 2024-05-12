import { View } from "./view.js";
import { Controller } from "./controller.js";
export declare enum DidHandleMessage {
    YES = 0,
    NO = 1
}
export declare class Model {
    private readonly core;
    private constructor();
    static create(view: View, controller: Controller, canvas: HTMLCanvasElement): Promise<Model>;
    set_scene(scene_name: string): Promise<DidHandleMessage>;
    resize(width: number, height: number): DidHandleMessage;
    turn_camera(drag_begin: {
        x: number;
        y: number;
    }, drag_end: {
        x: number;
        y: number;
    }): DidHandleMessage;
}
