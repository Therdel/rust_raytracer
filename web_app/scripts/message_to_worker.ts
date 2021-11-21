type MessageToWorker_Message =
    MessageToWorker_Init |
    MessageToWorker_SceneSelect |
    MessageToWorker_Resize |
    MessageToWorker_TurnCamera

class MessageToWorker_MessageWithBuffer {
    readonly type = "MessageToWorker_MessageWithBuffer"

    constructor(readonly buffer: ArrayBuffer,
                readonly message: MessageToWorker_Message) {
    }
}

class MessageToWorker_Init {
    readonly type = "MessageToWorker_Init"

    constructor(readonly index: number,
                readonly amount_workers: number,
                readonly scene_file: string,
                readonly width: number,
                readonly height: number) {
    }
}

class MessageToWorker_SceneSelect {
    readonly type = "MessageToWorker_SceneSelect"

    constructor(readonly scene_file: string) {
    }
}

class MessageToWorker_Resize {
    readonly type = "MessageToWorker_Resize"

    constructor(readonly width: number,
                readonly height: number) {
    }
}

class MessageToWorker_TurnCamera {
    readonly type = "MessageToWorker_TurnCamera"

    constructor(readonly drag_begin: { x: number; y: number },
                readonly drag_end: { x: number; y: number }) {
    }
}