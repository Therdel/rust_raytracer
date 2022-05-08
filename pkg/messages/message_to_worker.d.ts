export declare type Message = Init | SceneSelect | Resize | TurnCamera;
export declare class MessageWithBuffer {
    readonly buffer: SharedArrayBuffer;
    readonly message: Message;
    readonly type = "MessageToWorker_MessageWithBuffer";
    constructor(buffer: SharedArrayBuffer, message: Message);
}
export declare class Init {
    readonly index: number;
    readonly amount_workers: number;
    readonly scene_file: string;
    readonly width: number;
    readonly height: number;
    readonly type = "MessageToWorker_Init";
    constructor(index: number, amount_workers: number, scene_file: string, width: number, height: number);
}
export declare class SceneSelect {
    readonly scene_file: string;
    readonly type = "MessageToWorker_SceneSelect";
    constructor(scene_file: string);
}
export declare class Resize {
    readonly width: number;
    readonly height: number;
    readonly type = "MessageToWorker_Resize";
    constructor(width: number, height: number);
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
