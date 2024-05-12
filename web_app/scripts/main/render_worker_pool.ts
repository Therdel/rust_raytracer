import * as MessageToWorker from "../messages/message_to_worker.js"
import * as MessageFromWorker from "../messages/message_from_worker.js"

export interface RenderWorkerMessageDelegate {
    (message: MessageFromWorker.Message)
}

export class RenderWorkerPool {
    private message_delegate: RenderWorkerMessageDelegate
    private workers: Worker[]

    // starts the workers
    constructor(message_delegate: RenderWorkerMessageDelegate, amount_workers: number) {
        this.message_delegate = message_delegate
        this.init_workers(amount_workers)
    }

    private init_workers(amount_workers: number) {
        this.workers = []
        for (let index=0; index<amount_workers; ++index) {
            const worker = new Worker("pkg/worker/render_worker.js", {type:'module'});

            // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
            // source: https://stackoverflow.com/a/20279485
            worker.onmessage = (message) => this.on_worker_message(message);

            this.workers.push(worker)
        }
    }

    amount_workers(): number {
        return this.workers.length
    }

    post(index: number, message: MessageToWorker.Message) {
        const worker = this.workers[index];
        worker.postMessage(message);
    }

    private on_worker_message({data: message}: MessageEvent<MessageFromWorker.Message>) {
        this.message_delegate(message)
    }
}