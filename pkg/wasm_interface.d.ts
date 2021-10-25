/* tslint:disable */
/* eslint-disable */
/**
*/
export function main(): void;
/**
* @param {Uint8Array} canvas_u8
* @param {number} width
* @param {number} height
* @param {Uint8Array} scene
* @param {Uint8Array} mesh_obj
*/
export function render(canvas_u8: Uint8Array, width: number, height: number, scene: Uint8Array, mesh_obj: Uint8Array): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly render: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_start: () => void;
}

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {InitInput | Promise<InitInput>} module_or_path
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
