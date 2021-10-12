(async () => {
    let response = await fetch('wasm_interface.wasm');
    let bytes = await response.arrayBuffer();
    let { instance } = await WebAssembly.instantiate(bytes, { });

    console.log('Exported functions: ', instance.exports);

    let mod = instance;

    var canvas = document.getElementById('screen');
    const width = canvas.width;
    const height = canvas.height;
    if (canvas.getContext) {
        var ctx = canvas.getContext('2d');

        let byteSize = width * height * 4;
        var pointer = mod.exports.alloc( byteSize );

        var usub = new Uint8ClampedArray(mod.exports.memory.buffer, pointer, byteSize);
        var img = new ImageData(usub, width, height);

        //var start = null;
        function step(timestamp) {
            //var progress;
            //if (start === null) start = timestamp;
            //progress = timestamp - start;
            //if (progress > 100) {
                mod.exports.render(pointer, width, height);
                //console.log('time taken: ', time_taken);

                //start = timestamp

                window.requestAnimationFrame(draw);
            //} else {
            //    window.requestAnimationFrame(step);
            //}
        }

        function draw() {
            ctx.putImageData(img, 0, 0)
            //window.requestAnimationFrame(step);
        }

        function render() {
            mod.exports.render(pointer, width, height);
            ctx.putImageData(img, 0, 0)
            console.log('rendered');
        }

        //window.requestAnimationFrame(step);
        var button = document.getElementById("run-wasm");
        button.addEventListener("click", function(e) {
            window.requestAnimationFrame(step);
        });
    }
})();