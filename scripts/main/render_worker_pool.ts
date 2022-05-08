// /// <reference path="../message_to_worker.ts" />
// /// <reference path="../message_from_worker.ts" />
// /// <reference path="../messages/message_to_worker.ts" />
// /// <reference path="../messages/message_from_worker.ts" />

import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"

export interface RenderWorkerMessageDelegate {
    (message: MessageFromWorker.Message)
}

export class RenderWorkerPool {
    private message_delegate: RenderWorkerMessageDelegate
    private workers: Worker[]
    public worker_image_buffers: SharedArrayBuffer[]

    constructor(message_delegate: RenderWorkerMessageDelegate,
                canvas_width: number, canvas_height: number) {
        this.message_delegate = message_delegate

        let amount_workers;
        if (navigator.hardwareConcurrency) {
            amount_workers = navigator.hardwareConcurrency
        } else {
            amount_workers = 4
        }

        this.init_workers(amount_workers)
        this.configure_worker_image_buffers(canvas_width, canvas_height)
    }

    private init_workers(amount_workers: number) {
        this.workers = []
        for (let index=0; index<amount_workers; ++index) {
            // const worker = new Worker("pkg/worker/render_worker.js");
            const worker = new Worker("pkg/worker/render_worker.js", {type:'module'});

            // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
            // source: https://stackoverflow.com/a/20279485
            worker.onmessage = (message) => this.on_worker_message(message);

            this.workers.push(worker)
        }
    }

    // TODO: Find better place / abstraction
    configure_worker_image_buffers(width: number, height: number) {
        // this.worker_image_buffers = []
        const image_buf_size = width * height * 4
        this.worker_image_buffers = [new SharedArrayBuffer(image_buf_size)]
        // let image_buffer = new SharedArrayBuffer(image_buf_size)
        // for (let i = 0; i < this.amount_workers(); ++i) {
            // const image_buffer = new ArrayBuffer(image_buf_size)
        // }
    }

    shared_buffer(): SharedArrayBuffer {
        return this.worker_image_buffers[0]
    }

    amount_workers(): number {
        return this.workers.length
    }

    post(index: number, message: MessageToWorker.Message) {
        const worker = this.workers[index];

        // const buffer = this.worker_image_buffers[index]
        const buffer = this.shared_buffer()
        const message_with_buffer =
            new MessageToWorker.MessageWithBuffer(buffer, message)
        worker.postMessage(message_with_buffer)//, [message_with_buffer.buffer]);
    }

    private on_worker_message({data: message}: MessageEvent<MessageFromWorker.Message>) {
        // if (message.type == "MessageFromWorker_RenderResponse") {
        //     this.worker_image_buffers[message.index] = message.buffer
        // }

        this.message_delegate(message)
    }
}