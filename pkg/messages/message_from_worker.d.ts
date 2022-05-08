export declare type Message = Init | RenderResponse;
export declare class Init {
    readonly type = "MessageFromWorker_Init";
    constructor();
}
export declare class RenderResponse {
    readonly index: number;
    readonly type = "MessageFromWorker_RenderResponse";
    constructor(index: number);
}
