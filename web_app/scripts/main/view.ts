export class View {
    private worker_canvases: Array<HTMLCanvasElement>
    private canvas_resizer: HTMLDivElement
    private touchscreen: HTMLDivElement
    private label_time_measurement: HTMLLabelElement

    constructor(amount_workers: number) {
        this.worker_canvases = new Array<HTMLCanvasElement>(amount_workers)
        this.canvas_resizer = document.getElementById("canvas-resizer") as HTMLDivElement
        this.touchscreen = document.getElementById("touchscreen") as HTMLDivElement
        
        const canvas_container = document.getElementById("canvas-container") as HTMLDivElement
        for (var i=0; i<amount_workers; ++i) {
            const worker_canvas: HTMLCanvasElement = document.createElement('canvas')
            worker_canvas.id = `worker_canvas${i}`

            this.worker_canvases[i] = worker_canvas
            canvas_container.appendChild(worker_canvas)
        }

        this.label_time_measurement = document.getElementById("time-measurement") as HTMLLabelElement
    }

    get_canvas_resizer(): HTMLDivElement {
        return this.canvas_resizer
    }

    get_touchscreen(): HTMLDivElement {
        return this.touchscreen
    }

    get_offscreen_canvases_once(): Array<OffscreenCanvas> {
        const result = new Array<OffscreenCanvas>(this.worker_canvases.length)
        for (var index=0; index<this.worker_canvases.length; ++index) {
            const canvas = this.worker_canvases[index]
            const offscreen_canvas = canvas.transferControlToOffscreen()
            result[index] = offscreen_canvas
        }

        return result
    }

    display_render_duration(duration: number) {
        this.label_time_measurement.innerHTML = `Render time: ${duration.toFixed(0)} ms`;
    }

    display_rendering_state() {
        this.label_time_measurement.innerHTML = `Rendering...`
    }
}