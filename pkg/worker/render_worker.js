var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
import * as MessageFromWorker from "../messages/message_from_worker.js";
import init, { Renderer, wasm_main } from "../../pkg/web_app.js";
const SCENE_BASE_PATH = "../../res/scenes";
const CHEAT_MODEL_PATH = "../../res/models/santa.obj";
class RenderWorker {
    constructor(index, buffer, amount_workers, scene, width, height) {
        this.index = index;
        this.canvas_buffer = buffer;
        this.canvas_buffer_u8 = new Uint8Array(this.canvas_buffer);
        this.amount_workers = amount_workers;
        this.width = width;
        this.height = height;
        this.renderer = new Renderer(width, height, scene, RenderWorker.cheat_obj_file);
    }
    static getInstance() {
        return RenderWorker.instance;
    }
    static init_cheat_obj() {
        return __awaiter(this, void 0, void 0, function* () {
            this.cheat_obj_file = yield fetch_into_array(CHEAT_MODEL_PATH);
        });
    }
    static init(message) {
        return __awaiter(this, void 0, void 0, function* () {
            yield init_wasm();
            // FIXME: These fetches are not done by the workers in parallel.
            //        Move to main?
            yield RenderWorker.init_cheat_obj();
            const { index, buffer, amount_workers, scene_file: scene_file, width, height } = message;
            const scene_url = SCENE_BASE_PATH + '/' + scene_file;
            const scene = yield fetch_into_array(scene_url);
            RenderWorker.instance = new RenderWorker(index, buffer, amount_workers, scene, width, height);
        });
    }
    static scene_select(_a) {
        return __awaiter(this, arguments, void 0, function* ({ scene_file: scene_file }) {
            const instance = RenderWorker.getInstance();
            const scene_url = SCENE_BASE_PATH + '/' + scene_file;
            const scene = yield fetch_into_array(scene_url);
            instance.renderer = new Renderer(instance.width, instance.height, scene, this.cheat_obj_file);
        });
    }
    static resize({ width, height, buffer }) {
        const instance = RenderWorker.getInstance();
        instance.width = width;
        instance.height = height;
        instance.canvas_buffer = buffer;
        instance.canvas_buffer_u8 = new Uint8Array(instance.canvas_buffer);
        instance.renderer.resize_screen(width, height);
    }
    static turn_camera(message) {
        const { drag_begin: { x: begin_x, y: begin_y }, drag_end: { x: end_x, y: end_y } } = message;
        RenderWorker.instance.renderer.turn_camera(begin_x, begin_y, end_x, end_y);
    }
    static render() {
        const instance = RenderWorker.getInstance();
        const y_offset = instance.index;
        const row_jump = instance.amount_workers;
        instance.renderer.render_interlaced(instance.canvas_buffer_u8, y_offset, row_jump);
    }
    static index() {
        return RenderWorker.instance.index;
    }
}
function init_wasm() {
    return __awaiter(this, void 0, void 0, function* () {
        // Load wasm file, run its entry point
        yield init();
        wasm_main();
    });
}
function fetch_into_array(path) {
    return __awaiter(this, void 0, void 0, function* () {
        let array_buffer = yield (yield fetch(path)).arrayBuffer();
        return new Uint8Array(array_buffer);
    });
}
function on_message(_a) {
    return __awaiter(this, arguments, void 0, function* ({ data: message }) {
        console.debug(`Worker:\tReceived '${message.type}'`);
        if (message.type === "MessageToWorker_Init") {
            const worker_init_start = performance.now();
            yield RenderWorker.init(message);
            const worker_init_duration = (performance.now() - worker_init_start).toFixed(0);
            console.debug(`Worker:\tinit took ${worker_init_duration}ms`);
        }
        else if (message.type === "MessageToWorker_SceneSelect") {
            yield RenderWorker.scene_select(message);
        }
        else if (message.type === "MessageToWorker_Resize") {
            RenderWorker.resize(message);
        }
        else if (message.type === "MessageToWorker_TurnCamera") {
            RenderWorker.turn_camera(message);
        }
        const worker_render_start = performance.now();
        RenderWorker.render();
        const worker_render_stop = performance.now() - worker_render_start;
        console.debug(`Worker:${RenderWorker.index()}\tResponding - Render time: ${worker_render_stop.toFixed(0)} ms`);
        const response = new MessageFromWorker.RenderResponse(RenderWorker.index());
        postMessage(response);
    });
}
onmessage = on_message;
const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds));
};
function init_worker() {
    return __awaiter(this, void 0, void 0, function* () {
        console.log(`Worker:\tstarted`);
        const init_message = new MessageFromWorker.Init();
        postMessage(init_message);
    });
}
init_worker();
