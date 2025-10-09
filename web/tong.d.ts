/* tslint:disable */
/* eslint-disable */
export class TongRepl {
  free(): void;
  [Symbol.dispose](): void;
  constructor();
  /**
   * Evaluate a line of Tong code and return the result as a string
   */
  eval(source: string): string;
  /**
   * Compile source to WASM and return as Uint8Array
   */
  static compile_wasm(source: string): Uint8Array;
  /**
   * Compile source to WAT text format
   */
  static compile_wat(source: string): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_tongrepl_free: (a: number, b: number) => void;
  readonly tongrepl_new: () => number;
  readonly tongrepl_eval: (a: number, b: number, c: number) => [number, number, number, number];
  readonly tongrepl_compile_wasm: (a: number, b: number) => [number, number, number, number];
  readonly tongrepl_compile_wat: (a: number, b: number) => [number, number, number, number];
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
