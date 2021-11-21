type MessageFromWorker_Message =
    MessageFromWorker_Init |
    MessageFromWorker_RenderResponse;

class MessageFromWorker_Init {
    readonly type = "MessageFromWorker_Init"

    constructor() {
    }
}

class MessageFromWorker_RenderResponse {
    readonly type = "MessageFromWorker_RenderResponse"

    constructor(readonly index: number,
                readonly buffer: ArrayBuffer) {
    }
}