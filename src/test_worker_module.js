

import {test_module_import} from "./test_module.js";

onmessage = ({ data }) => {
    postMessage(`hi from worker. Mainwindow said: [${data}]`);
    postMessage(`Calling imported function from module worker: ${test_module_import()}`)
    postMessage(`SharedArrayBuffer Test from worker: ${new SharedArrayBuffer(10)}`);
}