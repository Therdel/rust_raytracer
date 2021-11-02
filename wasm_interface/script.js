async function run() {
    let canvas = document.getElementById('screen');
    let button = document.getElementById("run-wasm");
    let label = document.getElementById('time-measurement');

    const width = canvas.width;
    const height = canvas.height;
    const canvas_buf_len = width * height * 4;
    let canvas_buf = new ArrayBuffer(canvas_buf_len);

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
    const worker_for_render = new Worker('worker_for_render.js');

    worker_for_render.onmessage = function (e) {
        console.log('Message received from worker');
        const { render_duration, buffer=canvas_buf } = e.data;
        label.innerHTML = `Render time: ${render_duration.toFixed(0)} ms`;
        canvas_buf = buffer;

        let canvas_image_data = new ImageData(new Uint8ClampedArray(canvas_buf), width, height);
        window.requestAnimationFrame(function () {
            ctx.putImageData(canvas_image_data, 0, 0);
        });
        button.disabled = false;
    };

    button.addEventListener("click", function () {
        button.disabled = true;

        label.innerHTML = `Rendering...`;
        let message_data = {
            buffer: canvas_buf,
            width: width,
            height: height,
        };
        worker_for_render.postMessage(message_data, [message_data.buffer]);
        console.log('Message posted to worker');
    });
}

run();