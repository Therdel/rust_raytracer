/// <reference path="../message_to_worker.ts" />
/// <reference path="../message_from_worker.ts" />

import {View} from "./view.js";
import {Controller} from "./controller.js";
import {RenderWorkerPool} from "./render_worker_pool.js";

export class Model {
    private readonly core: ModelCore

    constructor(view: View, controller: Controller, canvas: HTMLCanvasElement) {
        this.core = new ModelCore(view, controller, canvas)
    }

    scene_select(scene_file: string) {
        this.core.scene_select(scene_file)
    }

    resize(width: number,
           height: number) {
        this.core.resize(width, height)
    }

    turn_camera(drag_begin: { x: number, y: number },
                drag_end: { x: number, y: number }) {
        this.core.turn_camera(drag_begin, drag_end)
    }
}

class ModelCore {
    public readonly view: View
    public readonly controller: Controller

    private state: ModelState.State

    private readonly canvas: HTMLCanvasElement
    private readonly canvas_context: CanvasRenderingContext2D
    private image_data: ImageData

    public readonly render_worker_pool: RenderWorkerPool

    constructor(view: View, controller: Controller,
                canvas: HTMLCanvasElement) {
        this.view = view
        this.controller = controller

        this.state = new ModelState.InitPingPong(this)

        this.canvas = canvas
        this.canvas_context = canvas.getContext("2d")
        this.init_image_data()

        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const delegate = (message) => this.on_worker_message(message)
        this.render_worker_pool = new RenderWorkerPool(delegate, this.canvas.width, this.canvas.height)
    }

    transition_state(state: ModelState.State) {
        console.debug(`Model transition: ${this.state.state_name()} -> ${state.state_name()}`)
        this.state = state
    }

    init_image_data() {
        const [width, height] = [this.canvas.width, this.canvas.height]
        this.image_data = this.canvas_context.createImageData(width, height)
    }

    get_image_data() {
        return this.image_data
    }

    scene_select(scene_file: string) {
        this.state.scene_select(scene_file)
    }

    resize(width: number,
           height: number) {
        console.debug(`resize event`)
        this.state.resize(width, height)
    }

    turn_camera(drag_begin: { x: number, y: number },
                drag_end: { x: number, y: number }) {
        this.state.turn_camera(drag_begin, drag_end)
    }

    private on_worker_message(message: MessageFromWorker_Message) {
        this.state.on_message(message)
    }

    write_interlaced_worker_buffer_into_image_data(index: number, buffer: ArrayBuffer) {
        const [width, height] = [this.canvas.width, this.canvas.height]
        const row_jump = this.render_worker_pool.amount_workers();
        const row_len_bytes = width * 4;

        for (let y = index; y < height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes;
            const row_dst = new Uint8Array(this.image_data.data.buffer, row_begin_offset, row_len_bytes);
            const row_src = new Uint8Array(buffer, row_begin_offset, row_len_bytes);
            row_dst.set(row_src);
        }
    }
}

namespace ModelState {

    enum DidHandleMessage {
        YES,
        NO
    }
    export interface State {
        scene_select(scene_file: string)

        resize(width: number, height: number)

        turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number })

        on_message(message: MessageFromWorker_Message)

        state_name(): string
    }

    abstract class AbstractState implements State {
        protected model: ModelCore

        constructor(model: ModelCore) {
            this.model = model
        }

        scene_select(scene_file: string) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle scene_select(${scene_file})`)
        }

        resize(width: number, height: number) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle resize(`, {width, height}, `)`)
        }

        turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number }) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle turn_camera(`, {drag_begin, drag_end}, `)`)
        }

        on_message(message: MessageFromWorker_Message) {
            const result = this.on_message_impl(message)
            if (result == DidHandleMessage.NO) {
                console.error(`ModelCore<${this.state_name()}>: Didn't handle message:`, message.constructor.name)
            }
        }

        protected on_message_impl(message: MessageFromWorker_Message): DidHandleMessage {
            return DidHandleMessage.NO
        }

        abstract state_name(): string
    }

    export class InitPingPong extends AbstractState {
        worker_responses: number = 0

        constructor(model: ModelCore) {
            super(model);
        }

        private send_init_and_start_first_render() {
            const amount_workers = this.model.render_worker_pool.amount_workers()
            const canvas_size = this.model.controller.get_current_canvas_size()
            for (let index=0; index<amount_workers; ++index) {
                const message = new MessageToWorker_Init(
                    index,
                    amount_workers,
                    this.model.controller.get_current_scene_file(),
                    canvas_size.width,
                    canvas_size.height
                )
                this.model.render_worker_pool.post(index, message)
            }
            this.model.transition_state(new Rendering(this.model))
        }

        on_message_impl(message: MessageFromWorker_Message): DidHandleMessage {
            if (message.type == "MessageFromWorker_Init") {
                this.worker_responses += 1
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    this.send_init_and_start_first_render()
                }
                return DidHandleMessage.YES
            }
            return DidHandleMessage.NO
        }

        state_name(): string {
            return this.constructor.name;
        }
    }

    class Rendering extends AbstractState {
        worker_responses: number = 0
        time_start: number

        constructor(model: ModelCore) {
            super(model)
            this.model.view.display_rendering_state()
            this.time_start = performance.now()
        }

        on_message_impl(message: MessageFromWorker_Message): DidHandleMessage {
            if (message.type == "MessageFromWorker_RenderResponse") {
                this.model.write_interlaced_worker_buffer_into_image_data(message.index, message.buffer)
                this.model.view.update_canvas(this.model.get_image_data())

                this.worker_responses += 1
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    this.model.transition_state(new AcceptUserControl(this.model))
                    this.display_render_time()
                }
                return DidHandleMessage.YES
            }
            return DidHandleMessage.NO
        }

        private display_render_time() {
            const duration = performance.now() - this.time_start
            this.model.view.display_render_duration(duration)
        }

        state_name(): string {
            return this.constructor.name;
        }
    }

    class AcceptUserControl extends AbstractState {
        constructor(model: ModelCore) {
            super(model);
            this.model.controller.activate_controls()
        }

        private transition_to_rendering() {
            this.model.controller.deactivate_controls()
            this.model.transition_state(new Rendering(this.model))
        }

        private post_all(message: MessageToWorker_Message) {
            const amount_workers = this.model.render_worker_pool.amount_workers()
            for (let index=0; index<amount_workers; ++index) {
                this.model.render_worker_pool.post(index, message)
            }
        }

        resize(width: number, height: number) {
            const message = new MessageToWorker_Resize(width, height)
            this.model.init_image_data()
            this.model.render_worker_pool.configure_worker_image_buffers(width, height)

            this.post_all(message)
            this.transition_to_rendering()
        }

        scene_select(scene_file: string) {
            const message = new MessageToWorker_SceneSelect(scene_file)
            this.post_all(message)
            this.transition_to_rendering()
        }

        turn_camera(drag_begin: { x: number; y: number },
                    drag_end: { x: number; y: number }) {
            const message = new MessageToWorker_TurnCamera(drag_begin, drag_end)

            console.log("Posting turn_camera: ", message)
            this.post_all(message)
            this.transition_to_rendering()
        }

        state_name(): string {
            return this.constructor.name;
        }
    }
}