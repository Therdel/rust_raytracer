export type Message = Init | SetScene | Resize | TurnCamera;
export declare class Init {
    readonly index: number;
    readonly canvas_buffer: SharedArrayBuffer;
    readonly amount_workers: number;
    readonly set_scene: SetScene;
    readonly width: number;
    readonly height: number;
    readonly type = "MessageToWorker_Init";
    constructor(index: number, canvas_buffer: SharedArrayBuffer, amount_workers: number, set_scene: SetScene, width: number, height: number);
}
export declare class SetScene {
    readonly scene_file_buffer: SharedArrayBuffer;
    readonly meshes: Map<string, SharedArrayBuffer>;
    readonly type = "MessageToWorker_SetScene";
    constructor(scene_file_buffer: SharedArrayBuffer, meshes: Map<string, SharedArrayBuffer>);
}
export declare class Resize {
    readonly width: number;
    readonly height: number;
    readonly buffer: SharedArrayBuffer;
    readonly type = "MessageToWorker_Resize";
    constructor(width: number, height: number, buffer: SharedArrayBuffer);
}
export declare class TurnCamera {
    readonly drag_begin: {
        x: number;
        y: number;
    };
    readonly drag_end: {
        x: number;
        y: number;
    };
    readonly type = "MessageToWorker_TurnCamera";
    constructor(drag_begin: {
        x: number;
        y: number;
    }, drag_end: {
        x: number;
        y: number;
    });
}
