import * as MessageToWorker from "../messages/message_to_worker.js";
import * as MessageFromWorker from "../messages/message_from_worker.js";
export interface RenderWorkerMessageDelegate {
    (message: MessageFromWorker.Message): any;
}
export declare class RenderWorkerPool {
    private message_delegate;
    private workers;
    worker_image_buffers: SharedArrayBuffer[];
    constructor(message_delegate: RenderWorkerMessageDelegate, canvas_width: number, canvas_height: number);
    private init_workers;
    configure_worker_image_buffers(width: number, height: number): void;
    shared_buffer(): SharedArrayBuffer;
    amount_workers(): number;
    post(index: number, message: MessageToWorker.Message): void;
    private on_worker_message;
}
