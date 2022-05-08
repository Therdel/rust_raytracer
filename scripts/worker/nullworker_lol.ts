import {test_get_string} from "test_module.js"

console.log("Nullworker init");
console.log(`Nullworker imported function: ${test_get_string()}`);
onmessage = async ({ data }) => {
    const { message } = data
    console.debug(`Nullworker: Received '${message}'`);
}