function getCanvasImageData(canvas) {
    const width = canvas.width;
    const height = canvas.height;
    const canvas_buf_len = width * height * 4;
    const canvas_buf = new Uint8ClampedArray(canvas_buf_len);

    return new ImageData(canvas_buf, width, height);
}

async function run() {
    let canvas = document.getElementById('screen');
    let button = document.getElementById("run-wasm");
    let label = document.getElementById('time-measurement');

    if (!window.Worker) {
        alert('Your browser doesn\'t support web workers.');
        console.log('Your browser doesn\'t support web workers.');
        return;
    } else if (!canvas.getContext) {
        alert('Couldn\'t get canvas.getContext');
        console.log('Couldn\'t get canvas.getContext');
        return;
    }
    let ctx = canvas.getContext('2d');
    // TODO try without copying it here
    const canvas_image_data = getCanvasImageData(canvas);
    const worker_for_render = new Worker('worker_for_render.js');

    worker_for_render.onmessage = function (e) {
        console.log('Message received from worker');
        const [render_duration, canvas_image_data] = e.data;
        label.innerHTML = `Render time: ${render_duration.toFixed(0)} ms`;

        window.requestAnimationFrame(function () {
            ctx.putImageData(canvas_image_data, 0, 0);
        });
        button.disabled = false;
    }

    button.addEventListener("click", function () {
        button.disabled = true;

        label.innerHTML = `Rendering...`;
        worker_for_render.postMessage(canvas_image_data);
        console.log('Message posted to worker');
    });
}

run();