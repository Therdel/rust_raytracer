declare type MessageFromWorker_Message = MessageFromWorker_Init | MessageFromWorker_RenderResponse;
declare class MessageFromWorker_Init {
    readonly type = "MessageFromWorker_Init";
    constructor();
}
declare class MessageFromWorker_RenderResponse {
    readonly index: number;
    readonly type = "MessageFromWorker_RenderResponse";
    constructor(index: number);
}
