window.onload = async () => {
    let response = await fetch('../target/wasm32-unknown-unknown/release/wasm_interface.wasm');
    let bytes = await response.arrayBuffer();
    let { instance } = await WebAssembly.instantiate(bytes, { });

    let mod = instance;

    var canvas = document.getElementById('screen');
    let label = document.getElementById('time-measurement');

    if (canvas.getContext) {
        var ctx = canvas.getContext('2d');

        const width = canvas.width;
        const height = canvas.height;
        let byteSize = width * height * 4;
        let pointer = mod.exports.alloc( byteSize );

        let usub = new Uint8ClampedArray(mod.exports.memory.buffer, pointer, byteSize);
        let img = new ImageData(usub, width, height);

        function render() {
            let startTime = performance.now();
            mod.exports.render(pointer, width, height);
            let endTime = performance.now();

            ctx.putImageData(img, 0, 0);

            label.innerHTML = `Render time: ${(endTime - startTime).toFixed(0)} ms`;
        }

        let button = document.getElementById("run-wasm");
        button.addEventListener("click", function(e) {
            label.innerHTML = `Rendering...`;
            window.requestAnimationFrame(render);
        });
    }
};