export class MessageWithBuffer {
    constructor(buffer, message) {
        this.buffer = buffer;
        this.message = message;
        this.type = "MessageToWorker_MessageWithBuffer";
    }
}
export class Init {
    constructor(index, amount_workers, scene_file, width, height) {
        this.index = index;
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
    constructor(width, height) {
        this.width = width;
        this.height = height;
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
