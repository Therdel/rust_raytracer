class MessageFromWorker_Init {
    constructor() {
        this.type = "MessageFromWorker_Init";
    }
}
class MessageFromWorker_RenderResponse {
    // constructor(readonly index: number,
    //     readonly buffer: SharedArrayBuffer) {
    // }
    constructor(index) {
        this.index = index;
        this.type = "MessageFromWorker_RenderResponse";
    }
}
