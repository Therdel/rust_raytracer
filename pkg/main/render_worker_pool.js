/// <reference path="../message_to_worker.ts" />
/// <reference path="../message_from_worker.ts" />
export class RenderWorkerPool {
    constructor(message_delegate, canvas_width, canvas_height) {
        this.message_delegate = message_delegate;
        let amount_workers;
        if (navigator.hardwareConcurrency) {
            amount_workers = navigator.hardwareConcurrency;
        }
        else {
            amount_workers = 4;
        }
        this.init_workers(amount_workers);
        this.configure_worker_image_buffers(canvas_width, canvas_height);
    }
    init_workers(amount_workers) {
        this.workers = [];
        for (let index = 0; index < amount_workers; ++index) {
            const worker = new Worker("pkg/worker/render_worker.js");
            // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
            // source: https://stackoverflow.com/a/20279485
            worker.onmessage = (message) => this.on_worker_message(message);
            this.workers.push(worker);
        }
    }
    // TODO: Find better place / abstraction
    configure_worker_image_buffers(width, height) {
        this.worker_image_buffers = [];
        const image_buf_size = width * height * 4;
        for (let i = 0; i < this.amount_workers(); ++i) {
            const image_buffer = new ArrayBuffer(image_buf_size);
            this.worker_image_buffers.push(image_buffer);
        }
    }
    amount_workers() {
        return this.workers.length;
    }
    post(index, message) {
        const worker = this.workers[index];
        const buffer = this.worker_image_buffers[index];
        const message_with_buffer = new MessageToWorker_MessageWithBuffer(buffer, message);
        worker.postMessage(message_with_buffer, [message_with_buffer.buffer]);
    }
    on_worker_message({ data: message }) {
        if (message.type == "MessageFromWorker_RenderResponse") {
            this.worker_image_buffers[message.index] = message.buffer;
        }
        this.message_delegate(message);
    }
}
