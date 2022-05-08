declare type MessageToWorker_Message = MessageToWorker_Init | MessageToWorker_SceneSelect | MessageToWorker_Resize | MessageToWorker_TurnCamera;
declare class MessageToWorker_MessageWithBuffer {
    readonly buffer: SharedArrayBuffer;
    readonly message: MessageToWorker_Message;
    readonly type = "MessageToWorker_MessageWithBuffer";
    constructor(buffer: SharedArrayBuffer, message: MessageToWorker_Message);
}
declare class MessageToWorker_Init {
    readonly index: number;
    readonly amount_workers: number;
    readonly scene_file: string;
    readonly width: number;
    readonly height: number;
    readonly type = "MessageToWorker_Init";
    constructor(index: number, amount_workers: number, scene_file: string, width: number, height: number);
}
declare class MessageToWorker_SceneSelect {
    readonly scene_file: string;
    readonly type = "MessageToWorker_SceneSelect";
    constructor(scene_file: string);
}
declare class MessageToWorker_Resize {
    readonly width: number;
    readonly height: number;
    readonly type = "MessageToWorker_Resize";
    constructor(width: number, height: number);
}
declare class MessageToWorker_TurnCamera {
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
