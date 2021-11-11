// The worker has its own scope and no direct access to functions/objects of the
// global scope. We import the generated JS file to make `wasm_bindgen`
// available which we need to initialize our WASM code.
importScripts('./pkg/web_app.js');

const {main, Renderer} = wasm_bindgen;

async function init_wasm() {
    // Load the wasm file by awaiting the Promise returned by `wasm_bindgen`
    await wasm_bindgen('./pkg/web_app_bg.wasm');

    // Run main WASM entry point
    main();
}

async function fetch_into_array(path) {
    let array_buffer = await (await fetch(path)).arrayBuffer();
    // TODO: Throw error if file doesn't exist / is empty
    return new Uint8Array(array_buffer);
}

async function init_worker() {
    await init_wasm();

    let scene = await fetch_into_array('../res/scenes/scene_rust.json');
    let obj_file = await fetch_into_array('../res/models/santa.obj');

    // TODO: Init from init msg from main script
    let renderer = new Renderer(700, 700, scene, obj_file);

    function on_init(content) {
        const { width, height } = content;
        // TODO: Try creating once and passing to workers
        renderer = new Renderer(width, height, scene, obj_file);
    }

    function on_render(content) {
        const { index, buffer, amount_workers } = content;

        const y_offset = index;
        const row_jump = amount_workers;

        let startTime = performance.now();
        renderer.render_interlaced(new Uint8Array(buffer), y_offset, row_jump);
        let endTime = performance.now();

        console.log(`Worker#${index}: Posting message back to main script`);
        let content_out = {
            index,
            render_duration: endTime - startTime,
            buffer
        };
        postMessage({ is_init: false, content: content_out }, [content_out.buffer]);
    }

    onmessage = (msg) => {
        const { is_init, content } = msg.data;
        console.log(`Worker: Message received from main script. Init: ${is_init}`);

        if (is_init) {
            on_init(content);
        } else {
            on_render(content);
        }
    }
    // TODO: How to identify ourselves to the main script?
    postMessage({ is_init: true, content: null });
}

init_worker();