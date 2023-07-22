export class RenderWorkerPool {
    constructor(message_delegate) {
        this.message_delegate = message_delegate;
        let amount_workers;
        if (navigator.hardwareConcurrency) {
            amount_workers = navigator.hardwareConcurrency;
        }
        else {
            amount_workers = 4;
        }
        this.init_workers(amount_workers);
    }
    init_workers(amount_workers) {
        this.workers = [];
        for (let index = 0; index < amount_workers; ++index) {
            const worker = new Worker("pkg/worker/render_worker.js", { type: 'module' });
            // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
            // source: https://stackoverflow.com/a/20279485
            worker.onmessage = (message) => this.on_worker_message(message);
            this.workers.push(worker);
        }
    }
    amount_workers() {
        return this.workers.length;
    }
    post(index, message) {
        const worker = this.workers[index];
        worker.postMessage(message);
    }
    on_worker_message({ data: message }) {
        this.message_delegate(message);
    }
}
