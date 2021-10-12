(async () => {
        let response = await fetch('../target/wasm32-unknown-unknown/release/wasm_interface.wasm');
        let bytes = await response.arrayBuffer();
        let { instance } = await WebAssembly.instantiate(bytes, { });

        console.log('Exported functions: ', instance.exports);
        console.log('The answer is: ', instance.exports.color_sum());
      })();