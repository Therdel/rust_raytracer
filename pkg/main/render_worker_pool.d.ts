import * as MessageToWorker from "../messages/message_to_worker.js";
import * as MessageFromWorker from "../messages/message_from_worker.js";
export interface RenderWorkerMessageDelegate {
    (message: MessageFromWorker.Message): any;
}
export declare class RenderWorkerPool {
    private message_delegate;
    private workers;
    constructor(message_delegate: RenderWorkerMessageDelegate, amount_workers: number);
    private init_workers;
    amount_workers(): number;
    post(index: number, message: MessageToWorker.Message): void;
    private on_worker_message;
}
