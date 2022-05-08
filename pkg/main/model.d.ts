/// <reference path="../messages/message_to_worker.d.ts" />
/// <reference path="../messages/message_from_worker.d.ts" />
import { View } from "./view.js";
import { Controller } from "./controller.js";
export declare class Model {
    private readonly core;
    constructor(view: View, controller: Controller, canvas: HTMLCanvasElement);
    scene_select(scene_file: string): void;
    resize(width: number, height: number): void;
    turn_camera(drag_begin: {
        x: number;
        y: number;
    }, drag_end: {
        x: number;
        y: number;
    }): void;
}
