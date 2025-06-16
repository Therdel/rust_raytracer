export enum DidHandleMessage {
    YES,
    NO
}

export interface Model {
    set_scene(scene_name: string): Promise<DidHandleMessage>
    resize(width: number,
           height: number): Promise<DidHandleMessage>
    turn_camera(drag_begin: { x: number, y: number },
                drag_end:   { x: number, y: number }): Promise<DidHandleMessage>
}