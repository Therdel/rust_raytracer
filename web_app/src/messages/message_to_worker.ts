export class Message {
    constructor(readonly sequence: number,
                readonly worker_index: number,
                readonly payload: Payload) {
    }
}

export type Payload = Init |
                      Render |
                      SetScene |
                      Resize |
                      TurnCamera

export class Init {
    readonly type = "MessageToWorker_Init"

    constructor(readonly amount_workers: number,
                readonly resize: Resize) {
    }
}

export class Render {
    readonly type = "MessageToWorker_Render"
}

export class SetScene {
    readonly type = "MessageToWorker_SetScene"

    // TODO: communicate using e.g. IndexDB
    constructor(readonly scene_url_or_filename: string,
                readonly assets_serialized: Map<string, SharedArrayBuffer>) {
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