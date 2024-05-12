export class Init {
    constructor(index, canvas_buffer, amount_workers, set_scene, width, height) {
        this.index = index;
        this.canvas_buffer = canvas_buffer;
        this.amount_workers = amount_workers;
        this.set_scene = set_scene;
        this.width = width;
        this.height = height;
        this.type = "MessageToWorker_Init";
    }
}
export class SetScene {
    constructor(scene_file_buffer, meshes) {
        this.scene_file_buffer = scene_file_buffer;
        this.meshes = meshes;
        this.type = "MessageToWorker_SetScene";
    }
}
export class Resize {
    constructor(width, height, buffer) {
        this.width = width;
        this.height = height;
        this.buffer = buffer;
        this.type = "MessageToWorker_Resize";
    }
}
export class TurnCamera {
    constructor(drag_begin, drag_end) {
        this.drag_begin = drag_begin;
        this.drag_end = drag_end;
        this.type = "MessageToWorker_TurnCamera";
    }
}
