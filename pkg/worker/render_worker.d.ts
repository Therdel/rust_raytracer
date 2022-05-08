/// <reference path="../messages/message_to_worker.d.ts" />
/// <reference path="../messages/message_from_worker.d.ts" />
declare const SCENE_BASE_PATH = "../../res/scenes";
declare const CHEAT_MODEL_PATH = "../../res/models/santa.obj";
declare class RenderWorker {
    private index;
    private amount_workers;
    private width;
    private height;
    private renderer;
    private static instance;
    private static cheat_obj_file;
    private constructor();
    private static getInstance;
    static init_cheat_obj(): Promise<void>;
    static init({ index, amount_workers, scene_file: scene_file, width, height }: MessageToWorker_Init): Promise<void>;
    static scene_select({ scene_file: scene_file }: MessageToWorker_SceneSelect): Promise<void>;
    static resize({ width, height }: MessageToWorker_Resize): void;
    static turn_camera({ drag_begin: { x: begin_x, y: begin_y }, drag_end: { x: end_x, y: end_y } }: MessageToWorker_TurnCamera): void;
    static render(buffer: SharedArrayBuffer): void;
    static index(): number;
}
declare function init_wasm(): Promise<void>;
declare function fetch_into_array(path: any): Promise<Uint8Array>;
declare const sleep: (milliseconds: any) => Promise<unknown>;
declare function init_worker(): Promise<void>;
