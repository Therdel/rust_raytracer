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
import * as MessageToWorker from "../messages/message_to_worker.js";
export var DidHandleMessage;
(function (DidHandleMessage) {
    DidHandleMessage[DidHandleMessage["YES"] = 0] = "YES";
    DidHandleMessage[DidHandleMessage["NO"] = 1] = "NO";
})(DidHandleMessage || (DidHandleMessage = {}));
export class Model {
    constructor(core) {
        this.core = core;
    }
    static create(view, controller, canvas) {
        return __awaiter(this, void 0, void 0, function* () {
            const model_core = yield ModelCore.create(view, controller, canvas);
            return new Model(model_core);
        });
    }
    set_scene(scene_name) {
        return __awaiter(this, void 0, void 0, function* () {
            return yield this.core.set_scene(scene_name);
        });
    }
    resize(width, height) {
        return this.core.resize(width, height);
    }
    turn_camera(drag_begin, drag_end) {
        return this.core.turn_camera(drag_begin, drag_end);
    }
}
class ModelCore {
    constructor(view, controller, canvas) {
        this.view = view;
        this.controller = controller;
        this.state = undefined;
        this.canvas = canvas;
        this.canvas_context = canvas.getContext("2d");
        this.init_image_data();
        this.amount_workers = navigator.hardwareConcurrency ? navigator.hardwareConcurrency : 4;
        this.create_worker_image_buffers(this.canvas.width, this.canvas.height);
        this.render_worker_pool = undefined;
    }
    static create(view, controller, canvas) {
        return __awaiter(this, void 0, void 0, function* () {
            const model_core = new ModelCore(view, controller, canvas);
            const scene_file_name = controller.get_current_scene_file_name();
            yield model_core.fetch_scene_and_cache_meshes(scene_file_name);
            const init_set_scene = new MessageToWorker.SetScene(model_core.scene.file_buffer, model_core.mesh_cache);
            model_core.state = new ModelState.InitPingPong(model_core, init_set_scene);
            // start rendering
            const delegate = (message) => model_core.on_worker_message(message); // closure-wrap necessary, or else the this inside on_worker_message will refer to the calling worker source: https://stackoverflow.com/a/20279485
            model_core.render_worker_pool = new RenderWorkerPool(delegate, model_core.amount_workers);
            return model_core;
        });
    }
    set_scene(scene_name) {
        return __awaiter(this, void 0, void 0, function* () {
            yield this.fetch_scene_and_cache_meshes(scene_name);
            const set_scene = new MessageToWorker.SetScene(this.scene.file_buffer, this.mesh_cache);
            return this.state.set_scene(set_scene);
        });
    }
    resize(width, height) {
        return this.state.resize(width, height);
    }
    turn_camera(drag_begin, drag_end) {
        return this.state.turn_camera(drag_begin, drag_end);
    }
    fetch_scene_and_cache_meshes(scene_file_name) {
        return __awaiter(this, void 0, void 0, function* () {
            const scene_file_buffer = yield this.fetch_scene(scene_file_name);
            this.scene = { file_name: scene_file_name, file_buffer: scene_file_buffer };
            yield this.cache_scene_meshes(scene_file_buffer);
        });
    }
    fetch_scene(file_name) {
        return __awaiter(this, void 0, void 0, function* () {
            const SCENES_BASE_PATH = "res/scenes";
            const url = SCENES_BASE_PATH + '/' + file_name;
            let file_buffer_u8 = yield this.fetch_into_array(url);
            let file_buffer_shared = new SharedArrayBuffer(file_buffer_u8.byteLength);
            new Uint8Array(file_buffer_shared).set(file_buffer_u8);
            return file_buffer_shared;
        });
    }
    // parse the scene to cache its meshes
    cache_scene_meshes(scene_file_buffer) {
        return __awaiter(this, void 0, void 0, function* () {
            this.mesh_cache = new Map();
            const MODELS_BASE_PATH = "res/models";
            const scene_file_buffer_nonshared_for_decoding = new ArrayBuffer(scene_file_buffer.byteLength);
            const scene_file_buffer_u8 = new Uint8Array(scene_file_buffer_nonshared_for_decoding);
            scene_file_buffer_u8.set(new Uint8Array(scene_file_buffer));
            const scene_str = new TextDecoder().decode(scene_file_buffer_u8);
            const scene = JSON.parse(scene_str);
            // TODO validate scene, throw if of wrong format
            // "meshes": [
            //     {
            //         "name": "bunny",
            //         "file_name": "bunny.obj",
            //         "winding_order": "CounterClockwise",
            //         "material": "someShinyGreen"
            //     }
            // ]
            // const scene_format = {
            //     "meshes": [
            //         {
            //             "name": "bunny",
            //             "file_name": "bunny.obj",
            //             "winding_order": "CounterClockwise",
            //             "material": "someShinyGreen"
            //         }
            //     ]
            // }
            if ("meshes" in scene) {
                for (const mesh of scene.meshes) {
                    const mesh_file_name = mesh.file_name;
                    if (this.mesh_cache.has(mesh_file_name)) {
                        continue;
                    }
                    const mesh_url = MODELS_BASE_PATH + '/' + mesh_file_name;
                    let mesh_file_buffer_u8 = yield this.fetch_into_array(mesh_url);
                    let mesh_file_buffer_shared = new SharedArrayBuffer(mesh_file_buffer_u8.byteLength);
                    new Uint8Array(mesh_file_buffer_shared).set(mesh_file_buffer_u8);
                    this.mesh_cache.set(mesh_file_name, mesh_file_buffer_shared);
                    console.debug(`ModelCore cached new mesh: name=${mesh_file_name}`);
                }
            }
        });
    }
    transition_state(state) {
        console.debug(`Model:\ttransition: ${this.state.state_name()} -> ${state.state_name()}`);
        this.state = state;
    }
    init_image_data() {
        const [width, height] = [this.canvas.width, this.canvas.height];
        this.image_data = this.canvas_context.createImageData(width, height);
    }
    create_worker_image_buffers(width, height) {
        this.worker_image_buffers = [];
        const image_buf_size = width * height * 4;
        for (let i = 0; i < this.amount_workers; ++i) {
            const image_buffer = new SharedArrayBuffer(image_buf_size);
            this.worker_image_buffers.push(image_buffer);
        }
    }
    get_worker_buffer(index) {
        return this.worker_image_buffers[index];
    }
    get_image_data() {
        return this.image_data;
    }
    on_worker_message(message) {
        this.state.on_message(message);
    }
    write_interlaced_worker_buffer_into_image_data(index, src) {
        const dst = new Uint8Array(this.image_data.data.buffer);
        const y_offset = index;
        const row_jump = this.render_worker_pool.amount_workers();
        const [width, height] = [this.canvas.width, this.canvas.height];
        const row_len_bytes = width * 4;
        for (let y = y_offset; y < height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes;
            const row_dst = dst.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
            const row_src = src.subarray(row_begin_offset, row_begin_offset + row_len_bytes);
            row_dst.set(row_src);
        }
    }
    fetch_into_array(path) {
        return __awaiter(this, void 0, void 0, function* () {
            let array_buffer = yield (yield fetch(path)).arrayBuffer();
            return new Uint8Array(array_buffer);
        });
    }
}
var ModelState;
(function (ModelState) {
    class AbstractState {
        constructor(model) {
            this.model = model;
        }
        set_scene(message) {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle set_scene(${message})`);
            return DidHandleMessage.NO;
        }
        resize(width, height) {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle resize(`, { width, height }, `)`);
            return DidHandleMessage.NO;
        }
        turn_camera(drag_begin, drag_end) {
            console.log(`ModelCore<${this.state_name()}>: Didn't handle turn_camera(`, { drag_begin, drag_end }, `)`);
            return DidHandleMessage.NO;
        }
        on_message(message) {
            const result = this.on_message_impl(message);
            if (result == DidHandleMessage.NO) {
                console.error(`ModelCore<${this.state_name()}>: Didn't handle message:`, message.constructor.name);
            }
            return result;
        }
        on_message_impl(message) {
            return DidHandleMessage.NO;
        }
    }
    ModelState.AbstractState = AbstractState;
    class InitPingPong extends AbstractState {
        constructor(model, init_set_scene) {
            super(model);
            this.worker_responses = 0;
            this.init_set_scene = init_set_scene;
        }
        send_init_and_start_first_render() {
            const amount_workers = this.model.amount_workers;
            const canvas_size = this.model.controller.get_current_canvas_size();
            for (let index = 0; index < amount_workers; ++index) {
                const canvas_buffer = this.model.get_worker_buffer(index);
                const message = new MessageToWorker.Init(index, canvas_buffer, amount_workers, this.init_set_scene, canvas_size.width, canvas_size.height);
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
                const buffer = new Uint8Array(this.model.get_worker_buffer(message.index));
                this.model.write_interlaced_worker_buffer_into_image_data(message.index, buffer);
                this.worker_responses += 1;
                if (this.worker_responses == this.model.render_worker_pool.amount_workers()) {
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
            this.model.init_image_data();
            this.model.create_worker_image_buffers(width, height);
            const amount_workers = this.model.render_worker_pool.amount_workers();
            for (let index = 0; index < amount_workers; ++index) {
                const buffer = this.model.get_worker_buffer(index);
                const message = new MessageToWorker.Resize(width, height, buffer);
                this.model.render_worker_pool.post(index, message);
            }
            this.transition_to_rendering();
            return DidHandleMessage.YES;
        }
        set_scene(message) {
            this.post_all(message);
            this.transition_to_rendering();
            return DidHandleMessage.YES;
        }
        turn_camera(drag_begin, drag_end) {
            const message = new MessageToWorker.TurnCamera(drag_begin, drag_end);
            console.log("Posting turn_camera: ", message);
            this.post_all(message);
            this.transition_to_rendering();
            return DidHandleMessage.YES;
        }
        state_name() {
            return this.constructor.name;
        }
    }
})(ModelState || (ModelState = {}));
