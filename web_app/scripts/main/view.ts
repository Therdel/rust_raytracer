export class View {
    private canvas_context: CanvasRenderingContext2D
    private label_time_measurement: HTMLLabelElement

    constructor(canvas_context: CanvasRenderingContext2D) {
        this.canvas_context = canvas_context
        this.label_time_measurement = document.getElementById("time-measurement") as HTMLLabelElement
    }

    update_canvas(image_data: ImageData) {
        // window.requestAnimationFrame(() =>
            this.canvas_context.putImageData(image_data, 0, 0)
        // );
    }

    display_render_duration(duration: number) {
        this.label_time_measurement.innerHTML = `Render time: ${duration.toFixed(0)} ms`;
    }

    display_rendering_state() {
        this.label_time_measurement.innerHTML = `Rendering...`
    }
}