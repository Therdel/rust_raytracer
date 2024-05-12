export type Message = Init |
                      SetScene |
                      Resize |
                      TurnCamera

export class Init {
    readonly type = "MessageToWorker_Init"

    constructor(readonly index: number,
                readonly canvas_buffer: SharedArrayBuffer,
                readonly amount_workers: number,
                readonly set_scene: SetScene,
                readonly width: number,
                readonly height: number) {
    }
}

export class SetScene {
    readonly type = "MessageToWorker_SetScene"

    constructor(readonly scene_file_buffer: SharedArrayBuffer,
                readonly meshes: Map<string, SharedArrayBuffer>) {
    }
}

export class Resize {
    readonly type = "MessageToWorker_Resize"

    constructor(readonly width: number,
                readonly height: number,
                readonly buffer: SharedArrayBuffer) {
    }
}

export class TurnCamera {
    readonly type = "MessageToWorker_TurnCamera"

    constructor(readonly drag_begin: { x: number; y: number },
                readonly drag_end: { x: number; y: number }) {
    }
}