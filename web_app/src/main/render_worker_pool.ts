import * as MessageToWorker from "../messages/message_to_worker"
import * as MessageFromWorker from "../messages/message_from_worker"

export interface RenderWorkerMessageDelegate {
    (message: MessageFromWorker.Message): void
}

export class RenderWorkerPool {
    private message_delegate: RenderWorkerMessageDelegate
    private workers: Worker[]

    // starts the workers
    constructor(message_delegate: RenderWorkerMessageDelegate, amount_workers: number) {
        this.message_delegate = message_delegate
        this.workers = this.init_workers(amount_workers)
    }

    private init_workers(amount_workers: number): Worker[] {
        this.workers = []
        for (let index=0; index<amount_workers; ++index) {
            const worker = new Worker(
              new URL("../worker/render_worker", import.meta.url),
              {
                type: "module",
              }
            );

            // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
            // source: https://stackoverflow.com/a/20279485
            worker.onmessage = (message) => this.on_worker_message(message);

            this.workers.push(worker)
        }
        return this.workers
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