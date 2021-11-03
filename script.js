async function run() {
    let canvas = document.getElementById('screen');
    let button = document.getElementById("run-wasm");
    let label = document.getElementById('time-measurement');

    let render_start_time = 0;

    const width = canvas.width;
    const height = canvas.height;
    const canvas_buf_len = width * height * 4;
    const canvas_buf = new ArrayBuffer(canvas_buf_len);
    const canvas_image_data = new ImageData(new Uint8ClampedArray(canvas_buf), width, height);

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

    const amount_workers = navigator.hardwareConcurrency;
    console.log(`Amount workers: ${amount_workers}`);
    let workers_responded = 0;
    let workers = [];

    function draw_on_canvas_and_finish() {
        console.log(`Finishing with ${workers_responded} responses`);
        window.requestAnimationFrame(function () {
            ctx.putImageData(canvas_image_data, 0, 0);
        });
        const render_duration = performance.now() - render_start_time;
        label.innerHTML = `Render time: ${render_duration.toFixed(0)} ms`;
        button.disabled = false;
    }

    function write_worker_buffer_into_image_data(worker) {
        const row_jump = amount_workers;
        const row_len_bytes = width * 4;
        for (let y=worker.index; y<height; y += row_jump) {
            const row_begin_offset = y * row_len_bytes;
            const row_dst = new Uint8Array(canvas_buf, row_begin_offset, row_len_bytes);
            const row_src = new Uint8Array(worker.buffer, row_begin_offset, row_len_bytes);
            row_dst.set(row_src);
        }
    }

    function check_worker_init_and_activate_button() {
        ++workers_responded;
        if (workers_responded >= amount_workers) {
            button.disabled = false;
        }
    }

    // TODO: Worker init state { Init, Ready }

    function on_worker_message(event) {
        const { is_init, content } = event.data;
        if (is_init) {
            check_worker_init_and_activate_button();
            // TODO: When to send first msg, when is the worker ready to receive it?
                // TODO: How to answer to a worker when it doesn't know its idx yet?
            //worker.postMessage({ is_init: true, content: { width, height,}});
            // TODO: Await all init responses before activating the button
            //console.log(`Init Message posted to ${index}`);
            check_worker_init_and_activate_button();
        } else {
            const { index, render_duration, buffer } = content;
            let worker = workers[index];

            console.log(`Message received from worker #${worker.index}`);

            worker.buffer = buffer;
            write_worker_buffer_into_image_data(worker);

            workers_responded += 1;
            if (workers_responded >= amount_workers) {
                draw_on_canvas_and_finish();
            }
        }
    }

    // init workers
    for (let index=0; index<amount_workers; ++index) {
        const worker = new Worker('worker_for_render.js');
        worker.onmessage = on_worker_message;
        let buffer = new ArrayBuffer(canvas_buf_len);
        workers.push({
            index,
            worker,
            buffer,
        })
    }

    button.addEventListener("click", function () {
        button.disabled = true;
        workers_responded = 0;
        render_start_time = performance.now();

        label.innerHTML = `Rendering...`;
        for (let index=0; index<amount_workers; ++index) {
            const worker = workers[index];

            let content = {
                index,
                buffer: worker.buffer,
                amount_workers,
            };
            worker.worker.postMessage({ is_init: false, content }, [content.buffer]);

            console.log(`Message posted to worker #${worker.index}`);
        }
    });
}

run();