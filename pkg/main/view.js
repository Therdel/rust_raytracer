export class View {
    constructor(canvas) {
        this.canvas = canvas;
        this.canvas_context = canvas.getContext("2d");
        this.label_time_measurement = document.getElementById("time-measurement");
    }
    update_canvas(image_data) {
        window.requestAnimationFrame(() => this.canvas_context.putImageData(image_data, 0, 0));
    }
    display_render_duration(duration) {
        this.label_time_measurement.innerHTML = `Render time: ${duration.toFixed(0)} ms`;
    }
    display_rendering_state() {
        this.label_time_measurement.innerHTML = `Rendering...`;
    }
}
