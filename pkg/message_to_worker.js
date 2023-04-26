class MessageToWorker_Init {
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
class MessageToWorker_SceneSelect {
    constructor(scene_file) {
        this.scene_file = scene_file;
        this.type = "MessageToWorker_SceneSelect";
    }
}
class MessageToWorker_Resize {
    constructor(width, height, buffer) {
        this.width = width;
        this.height = height;
        this.buffer = buffer;
        this.type = "MessageToWorker_Resize";
    }
}
class MessageToWorker_TurnCamera {
    constructor(drag_begin, drag_end) {
        this.drag_begin = drag_begin;
        this.drag_end = drag_end;
        this.type = "MessageToWorker_TurnCamera";
    }
}
