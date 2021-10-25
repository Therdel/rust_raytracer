// Use ES module import syntax to import functionality from the module
// that we have compiled.
//
// Note that the `default` import is an initialization function which
// will "boot" the module and make it ready to use. Currently browsers
// don't support natively imported WebAssembly as an ES module, but
// eventually the manual initialization won't be required!
import init, {render} from './pkg/wasm_interface.js';

async function run() {
    // First up we need to actually load the wasm file, so we use the
    // default export to inform it where the wasm file is located on the
    // server, and then we wait on the returned promise to wait for the
    // wasm to be loaded.
    //
    // It may look like this: `await init('./pkg/without_a_bundler_bg.wasm');`,
    // but there is also a handy default inside `init` function, which uses
    // `import.meta` to locate the wasm file relatively to js file.
    //
    // Note that instead of a string you can also pass in any of the
    // following things:
    //
    // * `WebAssembly.Module`
    //
    // * `ArrayBuffer`
    //
    // * `Response`
    //
    // * `Promise` which returns any of the above, e.g. `fetch("./path/to/wasm")`
    //
    // This gives you complete control over how the module is loaded
    // and compiled.
    //
    // Also note that the promise, when resolved, yields the wasm module's
    // exports which is the same as importing the `*_bg` module in other
    // modes

    // TODO: init panic handler
    await init();

    function print_arraybuffer(a) { console.log(new TextDecoder().decode(new Uint8Array(a))); }

    let scene_arraybuffer = await (await fetch('../res/scenes/scene_rust.json')).arrayBuffer();
    let scene = new Uint8Array(scene_arraybuffer);

    let obj_file_arraybuffer = await (await fetch('../res/models/santa.obj')).arrayBuffer();
    let obj_file = new Uint8Array(obj_file_arraybuffer);

    let canvas = document.getElementById('screen');
    let button = document.getElementById("run-wasm");
    let label = document.getElementById('time-measurement');

    if (canvas.getContext) {
        let ctx = canvas.getContext('2d');

        let width = canvas.width;
        let height = canvas.height;
        let canvas_buf_len = width * height * 4;
        let canvas_buf = new Uint8ClampedArray(canvas_buf_len);
        let canvas_img = new ImageData(canvas_buf, width, height);

        function render_js() {
            let startTime = performance.now();
            // TODO: Stop blocking main thread
            render(canvas_buf, width, height, scene, obj_file);
            let endTime = performance.now();

            ctx.putImageData(canvas_img, 0, 0);

            label.innerHTML = `Render time: ${(endTime - startTime).toFixed(0)} ms`;
            button.addEventListener("click", button_listener);
        }

        function button_listener (e) {
            label.innerHTML = `Rendering...`;
            button.addEventListener("click", function () {});
            // force redraw
            setTimeout(function () {
                window.requestAnimationFrame(render_js);
            }, 1);

        }
        button.addEventListener("click", button_listener);
    }
}

window.onload = run;