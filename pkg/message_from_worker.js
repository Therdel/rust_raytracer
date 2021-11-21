class MessageFromWorker_Init {
    constructor() {
        this.type = "MessageFromWorker_Init";
    }
}
class MessageFromWorker_RenderResponse {
    constructor(index, buffer) {
        this.index = index;
        this.buffer = buffer;
        this.type = "MessageFromWorker_RenderResponse";
    }
}
