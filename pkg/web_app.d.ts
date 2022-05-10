/* tslint:disable */
/* eslint-disable */
/**
*/
export function main(): void;
/**
* @param {number} num_threads
* @returns {Promise<any>}
*/
export function initThreadPool(num_threads: number): Promise<any>;
/**
* @param {number} receiver
*/
export function wbg_rayon_start_worker(receiver: number): void;
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
/**
* @param {number} width
* @param {number} height
*/
  resize_screen(width: number, height: number): void;
/**
* @param {number} drag_begin_x
* @param {number} drag_begin_y
* @param {number} drag_end_x
* @param {number} drag_end_y
*/
  turn_camera(drag_begin_x: number, drag_begin_y: number, drag_end_x: number, drag_end_y: number): void;
}
/**
*/
export class wbg_rayon_PoolBuilder {
  free(): void;
/**
* @returns {number}
*/
  numThreads(): number;
/**
* @returns {number}
*/
  receiver(): number;
/**
*/
  build(): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly main: () => void;
  readonly __wbg_renderer_free: (a: number) => void;
  readonly renderer_new: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
  readonly renderer_render: (a: number, b: number, c: number) => void;
  readonly renderer_render_interlaced: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly renderer_resize_screen: (a: number, b: number, c: number) => void;
  readonly renderer_turn_camera: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbg_wbg_rayon_poolbuilder_free: (a: number) => void;
  readonly wbg_rayon_poolbuilder_numThreads: (a: number) => number;
  readonly wbg_rayon_poolbuilder_receiver: (a: number) => number;
  readonly wbg_rayon_poolbuilder_build: (a: number) => void;
  readonly wbg_rayon_start_worker: (a: number) => void;
  readonly initThreadPool: (a: number) => number;
  readonly memory: WebAssembly.Memory;
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
* @param {WebAssembly.Memory} maybe_memory
*
* @returns {Promise<InitOutput>}
*/
export default function init (module_or_path?: InitInput | Promise<InitInput>, maybe_memory?: WebAssembly.Memory): Promise<InitOutput>;
