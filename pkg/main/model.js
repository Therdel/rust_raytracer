// /// <reference path="../message_to_worker.ts" />
// /// <reference path="../message_from_worker.ts" />
// /// <reference path="../messages/message_to_worker.ts" />
// /// <reference path="../messages/message_from_worker.ts" />
// /// <reference types="../../pkg/web_app" />
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import { RenderWorkerPool } from "./render_worker_pool.js";
import init, { main } from "../../pkg/web_app.js";
import * as MessageToWorker from "../messages/message_to_worker.js";
function init_wasm() {
    return __awaiter(this, void 0, void 0, function* () {
        // // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
        // await wasm_bindgen('pkg/web_app_bg.wasm');
        // //await wasm_bindgen('pkg/web_app_bg.wasm');
        // // Run main WASM entry point
        // wasm_bindgen.main();
        // Load the wasm file
        yield init();
        // Run main WASM entry point
        main();
    });
}
init_wasm();
export class Model {
    constructor(view, controller, canvas) {
        this.core = new ModelCore(view, controller, canvas);
    }
    scene_select(scene_file) {
        this.core.scene_select(scene_file);
    }
    resize(width, height) {
        this.core.resize(width, height);
    }
    turn_camera(drag_begin, drag_end) {
        this.core.turn_camera(drag_begin, drag_end);
    }
}
class ModelCore {
    constructor(view, controller, canvas) {
        this.view = view;
        this.controller = controller;
        this.state = new ModelState.InitPingPong(this);
        this.canvas = canvas;
        this.canvas_context = canvas.getContext("2d");
        this.init_image_data();
        // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker
        // source: https://stackoverflow.com/a/20279485
        const delegate = (message) => this.on_worker_message(message);
        this.render_worker_pool = new RenderWorkerPool(delegate, this.canvas.width, this.canvas.height);
    }
    transition_state(state) {
        console.debug(`Model transition: ${this.state.state_name()} -> ${state.state_name()}`);
        this.state = state;
    }
    init_image_data() {
        const [width, height] = [this.canvas.width, this.canvas.height];
        this.image_data = this.canvas_context.createImageData(width, height);
    }
    get_image_data() {
        return this.image_data;
    }
    scene_select(scene_file) {
        this.state.scene_select(scene_file);
    }
    resize(width, height) {
        console.debug(`resize event`);
        this.state.resize(width, height);
    }
    turn_camera(drag_begin, drag_end) {
        this.state.turn_camera(drag_begin, drag_end);
    }
    on_worker_message(message) {
        this.state.on_message(message);
    }
    write_interlaced_worker_buffer_into_image_data(index, buffer) {
        const dst = new Uint8Array(this.image_data.data.buffer);
        const src = new Uint8Array(buffer);
        const y_offset = index;
        const row_jump = this.render_worker_pool.amount_workers();
        const [width, height] = [this.canvas.width, this.canvas.height];
        // wasm_bindgen.put_buffer(dst, src, y_offset, row_jump, width, height)
        // put_buffer(dst, src, y_offset, row_jump, width, height)
        const row_len_bytes = width * 4;
        for (let y = index; y < height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes;
            const row_dst = new Uint8Array(this.image_data.data.buffer, row_begin_offset, row_len_bytes);
            const row_src = new Uint8Array(buffer, row_begin_offset, row_len_bytes);
            row_dst.set(row_src);
        }
    }
    // write_all_interlaced_worker_buffers_into_image_data() {
    //     const [width, height] = [this.canvas.width, this.canvas.height]
    //     const row_len_bytes = width * 4;
    //     const amount_buffers = this.render_worker_pool.amount_workers()
    //     for (let y = 0; y < height; ++y) {
    //         const row_begin_offset = y * row_len_bytes;
    //         const buffer_index = y % amount_buffers
    //         const buffer = this.render_worker_pool.worker_image_buffers[buffer_index]
    //         const row_dst = new Uint8Array(this.image_data.data.buffer, row_begin_offset, row_len_bytes);
    //         const row_src = new Uint8Array(buffer, row_begin_offset, row_len_bytes);
    //         row_dst.set(row_src);
    //     }
    // }
    // merge_interlaced_buffers_into_image_data() {
    //     wasm_bindgen
    // }
    overwrite_worker_buffer_into_image_data(buffer) {
        const dst = new Uint8Array(this.image_data.data.buffer);
        const src = new Uint8Array(buffer);
        dst.set(src);
    }
}
var ModelState;
(function (ModelState) {
    let DidHandleMessage;
    (function (DidHandleMessage) {
        DidHandleMessage[DidHandleMessage["YES"] = 0] = "YES";
        DidHandleMessage[DidHandleMessage["NO"] = 1] = "NO";
    })(DidHandleMessage || (DidHandleMessage = {}));
    class AbstractState {
        constructor(model) {
            this.model = model;
        }
        scene_select(scene_file) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle scene_select(${scene_file})`);
        }
        resize(width, height) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle resize(`, { width, height }, `)`);
        }
        turn_camera(drag_begin, drag_end) {
            console.error(`ModelCore<${this.state_name()}>: Didn't handle turn_camera(`, { drag_begin, drag_end }, `)`);
        }
        on_message(message) {
            const result = this.on_message_impl(message);
            if (result == DidHandleMessage.NO) {
                console.error(`ModelCore<${this.state_name()}>: Didn't handle message:`, message.constructor.name);
            }
        }
        on_message_impl(message) {
            return DidHandleMessage.NO;
        }
    }
    class InitPingPong extends AbstractState {
        constructor(model) {
            super(model);
            this.worker_responses = 0;
        }
        send_init_and_start_first_render() {
            const amount_workers = this.model.render_worker_pool.amount_workers();
            const canvas_size = this.model.controller.get_current_canvas_size();
            for (let index = 0; index < amount_workers; ++index) {
                const message = new MessageToWorker.Init(index, amount_workers, this.model.controller.get_current_scene_file(), canvas_size.width, canvas_size.height);
                this.model.render_worker_pool.post(index, message);
            }
            this.model.transition_state(new Rendering(this.model));
        }
        on_message_impl(message) {
            if (message.type == "MessageFromWorker_Init") {
                this.worker_responses += 1;
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    this.send_init_and_start_first_render();
                }
                return DidHandleMessage.YES;
            }
            return DidHandleMessage.NO;
        }
        state_name() {
            return this.constructor.name;
        }
    }
    ModelState.InitPingPong = InitPingPong;
    class Rendering extends AbstractState {
        constructor(model) {
            super(model);
            this.worker_responses = 0;
            this.model.view.display_rendering_state();
            this.time_start = performance.now();
        }
        on_message_impl(message) {
            if (message.type == "MessageFromWorker_RenderResponse") {
                // const is_first_response = this.worker_responses == 0
                // if (is_first_response) {
                //     this.model.overwrite_worker_buffer_into_image_data(message.buffer)
                // } else {
                //     this.model.write_interlaced_worker_buffer_into_image_data(message.index, message.buffer)
                // }
                // this.model.view.update_canvas(this.model.get_image_data())
                // this.model.overwrite_worker_buffer_into_image_data(this.model.render_worker_pool.shared_buffer());
                this.model.write_interlaced_worker_buffer_into_image_data(message.index, this.model.render_worker_pool.shared_buffer());
                // this.model.view.update_canvas(this.model.get_image_data())
                this.worker_responses += 1;
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
                    // this.model.write_all_interlaced_worker_buffers_into_image_data()
                    console.log("0000000000000000000000000000000022200");
                    // this.model.view.update_canvas(this.model.get_image_data())
                    //
                    // this.model.overwrite_worker_buffer_into_image_data(this.model.render_worker_pool.shared_buffer())
                    this.model.view.update_canvas(this.model.get_image_data());
                    this.model.transition_state(new AcceptUserControl(this.model));
                    this.display_render_time();
                }
                return DidHandleMessage.YES;
            }
            return DidHandleMessage.NO;
        }
        display_render_time() {
            const duration = performance.now() - this.time_start;
            this.model.view.display_render_duration(duration);
        }
        state_name() {
            return this.constructor.name;
        }
    }
    class AcceptUserControl extends AbstractState {
        constructor(model) {
            super(model);
            this.model.controller.activate_controls();
        }
        transition_to_rendering() {
            this.model.controller.deactivate_controls();
            this.model.transition_state(new Rendering(this.model));
        }
        post_all(message) {
            const amount_workers = this.model.render_worker_pool.amount_workers();
            for (let index = 0; index < amount_workers; ++index) {
                this.model.render_worker_pool.post(index, message);
            }
        }
        resize(width, height) {
            const message = new MessageToWorker.Resize(width, height);
            this.model.init_image_data();
            this.model.render_worker_pool.configure_worker_image_buffers(width, height);
            this.post_all(message);
            this.transition_to_rendering();
        }
        scene_select(scene_file) {
            const message = new MessageToWorker.SceneSelect(scene_file);
            this.post_all(message);
            this.transition_to_rendering();
        }
        turn_camera(drag_begin, drag_end) {
            const message = new MessageToWorker.TurnCamera(drag_begin, drag_end);
            console.log("Posting turn_camera: ", message);
            this.post_all(message);
            this.transition_to_rendering();
        }
        state_name() {
            return this.constructor.name;
        }
    }
})(ModelState || (ModelState = {}));
