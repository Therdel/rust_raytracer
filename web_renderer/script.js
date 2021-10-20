window.onload = async () => {
    let response = await fetch('../target/wasm32-unknown-unknown/release/wasm_interface.wasm');
    let bytes = await response.arrayBuffer();
    let { instance } = await WebAssembly.instantiate(bytes, { });
    let mod = instance;

    class WasmBuffer {
        constructor(arrayBuffer) {
            this.len = arrayBuffer.byteLength;
            this.buf_ptr = mod.exports.alloc( this.len );

            // copy file content into wasm buffer
            let buf = new Uint8Array(mod.exports.memory.buffer, this.buf_ptr, this.len);
            buf.set(new Uint8Array(arrayBuffer));
        }

        getLen() { return this.len; }
        getBufPtr() { return this.buf_ptr; }
    }

    function print_arraybuffer(a) { console.log(new TextDecoder().decode(new Uint8Array(a))); }

    let obj_sphere_arraybuffer = await (await fetch('../res/models/sphere_low.obj')).arrayBuffer();
    let obj_sphere = new WasmBuffer(obj_sphere_arraybuffer);

    let scene_arraybuffer = await (await fetch('../res/scenes/scene_rust.json')).arrayBuffer();
    let scene = new WasmBuffer(scene_arraybuffer);

    let canvas = document.getElementById('screen');
    let label = document.getElementById('time-measurement');

    if (canvas.getContext) {
        let ctx = canvas.getContext('2d');

        let width = canvas.width;
        let height = canvas.height;
        let canvas_buf_len = width * height * 4;
        let canvas_buf_ptr = mod.exports.alloc( canvas_buf_len );

        let canvas_buf = new Uint8ClampedArray(mod.exports.memory.buffer, canvas_buf_ptr, canvas_buf_len);
        let canvas_img = new ImageData(canvas_buf, width, height);

        function render() {
            let startTime = performance.now();
            mod.exports.render(canvas_buf_ptr, width, height,
                               scene.getBufPtr(), scene.getLen(),
                               obj_sphere.getBufPtr(), obj_sphere.getLen());
            let endTime = performance.now();

            ctx.putImageData(canvas_img, 0, 0);

            label.innerHTML = `Render time: ${(endTime - startTime).toFixed(0)} ms`;
        }

        let button = document.getElementById("run-wasm");
        button.addEventListener("click", function(e) {
            label.innerHTML = `Rendering...`;
            window.requestAnimationFrame(render);
        });
    }
};