export class Init {
    constructor(index, buffer, amount_workers, scene_file, width, height) {
        this.index = index;
        this.buffer = buffer;
        this.amount_workers = amount_workers;
        this.scene_file = scene_file;
        this.width = width;
        this.height = height;
        this.type = "MessageToWorker_Init";
    }
}
export class SceneSelect {
    constructor(scene_file) {
        this.scene_file = scene_file;
        this.type = "MessageToWorker_SceneSelect";
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
