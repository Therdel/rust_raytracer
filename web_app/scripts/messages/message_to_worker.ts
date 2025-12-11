export type Message = Init |
                      SetScene |
                      Resize |
                      TurnCamera |
                      AddMesh

export class Init {
    readonly type = "MessageToWorker_Init"

    constructor(readonly index: number,
                readonly amount_workers: number,
                readonly offscreen_canvas: OffscreenCanvas,
                readonly resize: Resize,
                readonly set_scene: SetScene) {
    }
}

export class SetScene {
    readonly type = "MessageToWorker_SetScene"

    constructor(readonly scene_file_buffer: SharedArrayBuffer,
                readonly meshes: Map<string, SharedArrayBuffer>) {
    }
}

export class Resize {
    readonly type = "MessageToWorker_Resize"

    constructor(readonly width: number,
                readonly height: number,
                readonly device_pixel_ratio: number /* ratio physical per css pixels */,
                readonly resolution_multiplier: number) {
    }
}

export class TurnCamera {
    readonly type = "MessageToWorker_TurnCamera"

    constructor(readonly drag_begin: { x: number; y: number },
                readonly drag_end: { x: number; y: number },
                readonly do_orbit: boolean) {
    }
}

export class AddMesh {
    readonly type = "MessageToWorker_AddMesh"

    constructor(readonly mesh_url: string,
                readonly mesh_file_buffer: SharedArrayBuffer) {
    }
}