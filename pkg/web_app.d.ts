declare namespace wasm_bindgen {
	/* tslint:disable */
	/* eslint-disable */
	/**
	*/
	export function main(): void;
	/**
	*/
	export class Renderer {
	  free(): void;
	/**
	* @param {number} width
	* @param {number} height
	* @param {Uint8Array} scene
	* @param {Uint8Array} mesh_obj
	*/
	  constructor(width: number, height: number, scene: Uint8Array, mesh_obj: Uint8Array);
	/**
	* @param {Uint8Array} canvas_u8
	*/
	  render(canvas_u8: Uint8Array): void;
	/**
	* @param {Uint8Array} canvas_u8
	* @param {number} y_offset
	* @param {number} row_jump
	*/
	  render_interlaced(canvas_u8: Uint8Array, y_offset: number, row_jump: number): void;
	}
	
}

declare type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

declare interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly main: () => void;
  readonly __wbg_renderer_free: (a: number) => void;
  readonly renderer_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly renderer_render: (a: number, b: number, c: number) => void;
  readonly renderer_render_interlaced: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbindgen_malloc: (a: number) => number;
  readonly __wbindgen_free: (a: number, b: number) => void;
  readonly __wbindgen_realloc: (a: number, b: number, c: number) => number;
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
declare function wasm_bindgen (module_or_path?: InitInput | Promise<InitInput>): Promise<InitOutput>;
