export class Message {
    constructor(readonly sequence: number,
                readonly worker_index: number) {
    }
}

export class Startup {
    readonly type = "MessageFromWorker_Startup"
}