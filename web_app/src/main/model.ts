export interface Model {
    render(): Promise<void>
    set_scene(scene_name: string): Promise<void>
    resize(width: number,
           height: number): Promise<void>
    turn_camera(drag_begin: { x: number, y: number },
                drag_end:   { x: number, y: number }): Promise<void>
}