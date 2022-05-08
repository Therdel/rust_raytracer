export declare class View {
    private canvas;
    private canvas_context;
    private label_time_measurement;
    constructor(canvas: HTMLCanvasElement);
    update_canvas(image_data: ImageData): void;
    display_render_duration(duration: number): void;
    display_rendering_state(): void;
}
