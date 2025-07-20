import * as MessageToWorker from "../messages/message_to_worker"
import * as MessageFromWorker from "../messages/message_from_worker"

enum MessageResult {
    Finished,
    Pending
}
type MessageHandler = (event: MessageEvent<MessageFromWorker.Message>) => MessageResult

export class RenderWorkerPool {
    private workers: Worker[]
    private first_pending_sequence_number: number
    private next_sequence_number: number
    private pending_message_handlers: (MessageHandler | undefined)[]

    private constructor(workers: Worker[]) {
        this.workers = workers
        this.workers.forEach(worker => {
            worker.onmessage = (event: MessageEvent<MessageFromWorker.Message>) => this.onMessage(event)
        })

        this.first_pending_sequence_number = 0
        this.next_sequence_number = 0
        this.pending_message_handlers = []
    }

    static async create(amount_workers: number): Promise<RenderWorkerPool> {
        const workers = await RenderWorkerPool.spawn_workers(amount_workers)
        return new RenderWorkerPool(workers)
    }

    private static async spawn_workers(amount_workers: number): Promise<Worker[]> {
        const workers: Worker[] = []

        // Create all workers
        for (let i = 0; i < amount_workers; i++) {
            const worker = new Worker(
                new URL("../worker/render_worker", import.meta.url),
                { type: "module" }
            )
            workers.push(worker)
        }

        // Wait for all workers to send their startup message
        await new Promise<void>((resolve) => {
            let started = 0

            const onMessage = ({ data: message }: MessageEvent<MessageFromWorker.Startup>) => {
                if (message.type === "MessageFromWorker_Startup") {
                    if (++started === amount_workers) {
                        // Remove listeners after all workers have started
                        workers.forEach(worker => worker.onmessage = null)
                        resolve()
                    }
                } else {
                    throw new Error(`Unexpected message: '${message}'`)
                }
            }

            workers.forEach(worker => worker.onmessage = onMessage)
        })

        return workers
    }

    async dispatch(mapFn: (workerIndex: number) => MessageToWorker.Payload,
                   reduceFn?: (workerIndex: number, isLastMessage: boolean) => void
    ): Promise<void> {
        const sequenceNumber = this.next_sequence_number++
        for (let i = 0; i < this.workers.length; i++) {
            const payload: MessageToWorker.Payload = mapFn(i)
            const message = new MessageToWorker.Message(sequenceNumber, i, payload)
            this.workers[i].postMessage(message)
        }

        return new Promise<void>((resolve) => {
            let receivedCount = 0
            const handler: MessageHandler = ({data: message}: MessageEvent<MessageFromWorker.Message>) => {
                const isLastMessage = ++receivedCount === this.workers.length;

                if (reduceFn) {
                    reduceFn(message.worker_index, isLastMessage)
                }

                if (isLastMessage) {
                    resolve()
                    return MessageResult.Finished
                } else {
                    return MessageResult.Pending
                }
            }
            this.pending_message_handlers.push(handler)
        })
    }

    private onMessage(event: MessageEvent<MessageFromWorker.Message>): void {
        const { sequence, worker_index } = event.data;
        if (sequence < this.first_pending_sequence_number) {
            throw new Error(`Received message with sequence number ${sequence} that's already been processed`);
        }
        if (sequence >= this.next_sequence_number) {
            throw new Error(`Received message with sequence number ${sequence} not yet sent`);
        }

        if (worker_index < 0 || worker_index >= this.workers.length) {
            throw new Error(`Received message from invalid worker index: ${worker_index}, expected 0 to ${this.workers.length - 1}`);
        }

        const handlerIndex = sequence - this.first_pending_sequence_number;
        if (handlerIndex >= this.pending_message_handlers.length) {
            throw new Error(`Received message with sequence number ${sequence} but no handler registered`);
        }
        const handler = this.pending_message_handlers[handlerIndex];
        if (!handler) {
            throw new Error(`Received message with sequence number ${sequence} but no handler registered`);
        }

        const result = handler(event);
        if (result === MessageResult.Finished) {
            this.pending_message_handlers[handlerIndex] = undefined;
            this.trimCompletedHandlers();
        }
    }

    // remove handlers of finished messages from the beginning
    private trimCompletedHandlers(): void {
        let count_completed = 0;
        for (let i = 0; i < this.pending_message_handlers.length; i++) {
            if (this.pending_message_handlers[i] === undefined) {
                ++count_completed;
            } else {
                break; // stop at the first non-finished handler
            }
        }
        this.first_pending_sequence_number += count_completed;
        this.pending_message_handlers = this.pending_message_handlers.slice(count_completed);
    }
}
