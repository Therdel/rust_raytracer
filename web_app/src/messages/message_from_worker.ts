export type Message = Init | RenderResponse;

export class Init {
    readonly type = "MessageFromWorker_Init"

    constructor() {
    }
}

export class RenderResponse {
    readonly type = "MessageFromWorker_RenderResponse"

    constructor(readonly index: number) {
    }
}