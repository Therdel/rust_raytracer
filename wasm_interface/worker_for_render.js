// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our WASM code.
importScripts('./pkg/wasm_interface.js');

const {main, Renderer} = wasm_bindgen;

async function init_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    await wasm_bindgen('./pkg/wasm_interface_bg.wasm');

    // Run main WASM entry point
    main();
}

async function fetch_into_array(path) {
    let array_buffer = await (await fetch(path)).arrayBuffer();
    return new Uint8Array(array_buffer);
}

async function init_worker() {
    await init_wasm();

    let scene = await fetch_into_array('../res/scenes/scene_rust.json');
    let obj_file = await fetch_into_array('../res/models/santa.obj');

    let renderer = null;

    onmessage = function (msg) {
        console.log('worker_for_render: Message received from main script');

        const { buffer, width, height } = msg.data;

        if (renderer === null) {
            renderer = new Renderer(width, height, scene, obj_file);
        }

        let startTime = performance.now();
        renderer.render(new Uint8Array(buffer));
        let endTime = performance.now();

        console.log('worker_for_render: Posting message back to main script');
        let message_data = {
            render_duration: endTime - startTime,
            buffer
        };
        postMessage(message_data, [message_data.buffer]);
    }
}

init_worker();