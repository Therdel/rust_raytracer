export class Init {
    constructor() {
        this.type = "MessageFromWorker_Init";
    }
}
export class RenderResponse {
    constructor(index) {
        this.index = index;
        this.type = "MessageFromWorker_RenderResponse";
    }
}
