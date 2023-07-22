import { View } from "./view.js";
import { Controller } from "./controller.js";
export declare enum DidHandleMessage {
    YES = 0,
    NO = 1
}
export declare class Model {
    private readonly core;
    constructor(view: View, controller: Controller, canvas: HTMLCanvasElement);
    scene_select(scene_file: string): DidHandleMessage;
    resize(width: number, height: number): DidHandleMessage;
    turn_camera(drag_begin: {
        x: number;
        y: number;
    }, drag_end: {
        x: number;
        y: number;
    }): DidHandleMessage;
}
