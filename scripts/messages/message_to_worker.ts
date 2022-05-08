export type Message = Init |
                      SceneSelect |
                      Resize |
                      TurnCamera

export class MessageWithBuffer {
    readonly type = "MessageToWorker_MessageWithBuffer"

    constructor(readonly buffer: SharedArrayBuffer,
                readonly message: Message) {
    }
}

export class Init {
    readonly type = "MessageToWorker_Init"

    constructor(readonly index: number,
                readonly amount_workers: number,
                readonly scene_file: string,
                readonly width: number,
                readonly height: number) {
    }
}

export class SceneSelect {
    readonly type = "MessageToWorker_SceneSelect"

    constructor(readonly scene_file: string) {
    }
}

export class Resize {
    readonly type = "MessageToWorker_Resize"

    constructor(readonly width: number,
                readonly height: number) {
    }
}

export class TurnCamera {
    readonly type = "MessageToWorker_TurnCamera"

    constructor(readonly drag_begin: { x: number; y: number },
                readonly drag_end: { x: number; y: number }) {
    }
}