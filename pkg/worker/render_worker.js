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
import * as MessageFromWorker from "../messages/message_from_worker.js";
import init, { Renderer, main } from "../../pkg/web_app.js";
// importScripts("../../pkg/web_app.js")
// importScripts("../messages/message_to_worker.js")
// importScripts("../messages/message_from_worker.js")
const SCENE_BASE_PATH = "../../res/scenes";
const CHEAT_MODEL_PATH = "../../res/models/santa.obj";
class RenderWorker {
    constructor(index, amount_workers, scene, width, height) {
        this.index = index;
        this.amount_workers = amount_workers;
        this.width = width;
        this.height = height;
        // this.renderer = new wasm_bindgen.Renderer(width,
        //                                           height,
        //                                           scene,
        //                                           RenderWorker.cheat_obj_file);
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
    static init({ index, amount_workers, scene_file: scene_file, width, height }) {
        return __awaiter(this, void 0, void 0, function* () {
            const scene_url = SCENE_BASE_PATH + '/' + scene_file;
            const scene = yield fetch_into_array(scene_url);
            RenderWorker.instance = new RenderWorker(index, amount_workers, scene, width, height);
        });
    }
    static scene_select({ scene_file: scene_file }) {
        return __awaiter(this, void 0, void 0, function* () {
            const instance = RenderWorker.getInstance();
            const scene_url = SCENE_BASE_PATH + '/' + scene_file;
            const scene = yield fetch_into_array(scene_url);
            // instance.renderer = new wasm_bindgen.Renderer(instance.width,
            //                                               instance.height,
            //                                               scene,
            //                                               this.cheat_obj_file)
            instance.renderer = new Renderer(instance.width, instance.height, scene, this.cheat_obj_file);
        });
    }
    static resize({ width, height }) {
        const instance = RenderWorker.getInstance();
        instance.width = width;
        instance.height = height;
        instance.renderer.resize_screen(width, height);
    }
    static turn_camera({ drag_begin: { x: begin_x, y: begin_y }, drag_end: { x: end_x, y: end_y } }) {
        RenderWorker.instance.renderer.turn_camera(begin_x, begin_y, end_x, end_y);
    }
    static render(buffer) {
        const instance = RenderWorker.getInstance();
        const y_offset = instance.index;
        const row_jump = instance.amount_workers;
        instance.renderer.render_interlaced(new Uint8Array(buffer), y_offset, row_jump);
    }
    static index() {
        return RenderWorker.instance.index;
    }
}
function init_wasm() {
    return __awaiter(this, void 0, void 0, function* () {
        // // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
        // await wasm_bindgen('../../pkg/web_app_bg.wasm');
        // // Run main WASM entry point
        // wasm_bindgen.main();
        // Load the wasm file
        yield init();
        // Run main WASM entry point
        main();
    });
}
function fetch_into_array(path) {
    return __awaiter(this, void 0, void 0, function* () {
        let array_buffer = yield (yield fetch(path)).arrayBuffer();
        return new Uint8Array(array_buffer);
    });
}
const sleep = (milliseconds) => {
    return new Promise(resolve => setTimeout(resolve, milliseconds));
};
function init_worker() {
    return __awaiter(this, void 0, void 0, function* () {
        console.log(`RenderWorker started`);
        const worker_init_start = performance.now();
        yield init_wasm();
        // FIXME: These fetches are not done by the workers in parallel.
        //        Move to main?
        yield RenderWorker.init_cheat_obj();
        onmessage = ({ data }) => __awaiter(this, void 0, void 0, function* () {
            const { buffer, message } = data;
            console.debug(`Worker: Received '${message.type}'`);
            if (message.type === "MessageToWorker_Init") {
                yield RenderWorker.init(message);
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
            RenderWorker.render(buffer);
            console.debug(`Worker: Responding`);
            const response = 
            // new MessageFromWorker_RenderResponse(RenderWorker.index(), buffer)
            new MessageFromWorker.RenderResponse(RenderWorker.index());
            postMessage(response); //, [buffer])
        });
        const init_message = new MessageFromWorker.Init();
        postMessage(init_message);
        const worker_init_duration = (performance.now() - worker_init_start).toFixed(0);
        console.debug(`RenderWorker init took ${worker_init_duration}ms`);
    });
}
init_worker();
