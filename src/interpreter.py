"""
Deprecated module: use the Rust implementation in rust/tong.
"""

raise RuntimeError("TONG Python interpreter retired; use the Rust CLI (see README.md)")
"""
TONG Language Interpreter and Runtime Engine
High-performance interpreter with JIT compilation support
"""

import builtins
import concurrent.futures
import multiprocessing
import os
import sys
import threading
import time
from abc import ABC, abstractmethod
from ctypes import (CDLL, c_int, c_uint32, c_uint8, c_char_p, c_float,
                    c_void_p, POINTER, Structure, byref)
from dataclasses import dataclass
from typing import Any, Dict, List, Optional, Union, Callable

from src.ast_nodes import *

class TongValue:
    """Base class for all runtime values"""
    def __init__(self, value: Any, type_name: str):
        self.value = value
        self.type_name = type_name

    def __repr__(self):
        return f"TongValue({self.value}, {self.type_name})"

class TongInteger(TongValue):
    """Represents 64-bit integer values in TONG."""
    def __init__(self, value: int):
        super().__init__(value, "i64")

class TongFloat(TongValue):
    """Represents 64-bit floating-point values in TONG."""
    def __init__(self, value: float):
        super().__init__(value, "f64")

class TongString(TongValue):
    """Represents string values in TONG."""
    def __init__(self, value: str):
        super().__init__(value, "String")

class TongBoolean(TongValue):
    """Represents boolean values in TONG."""
    def __init__(self, value: bool):
        super().__init__(value, "bool")

class TongArray(TongValue):
    """Represents array values in TONG."""
    def __init__(self, elements: List[TongValue]):
        super().__init__(elements, f"Array<{elements[0].type_name if elements else 'unknown'}>")

class TongNone(TongValue):
    """Represents None/null values in TONG."""
    def __init__(self):
        super().__init__(None, "None")

class TongFunction(TongValue):
    """Represents function values in TONG."""
    def __init__(self, func: Callable, signature: str):
        super().__init__(func, f"fn({signature})")

class TongExternal(TongValue):
    """Wrapper for external/native handles (e.g., SDL Window/Renderer)."""
    def __init__(self, value: Any, type_name: str):
        super().__init__(value, type_name)

class TongModule(TongValue):
    """Module value holding exported symbols as a dictionary."""
    def __init__(self, name: str, exports: Dict[str, TongValue]):
        super().__init__(exports, f"module<{name}>")
        self.name = name
        self.exports = exports

class RuntimeError(builtins.RuntimeError):
    """Runtime execution error (subclass of Python's RuntimeError)"""
    pass

class Environment:
    """Environment for variable and function bindings"""

    def __init__(self, parent: Optional['Environment'] = None):
        self.parent = parent
        self.bindings: Dict[str, TongValue] = {}

    def define(self, name: str, value: TongValue) -> None:
        """Define a new variable"""
        self.bindings[name] = value

    def get(self, name: str) -> TongValue:
        """Get a variable value"""
        if name in self.bindings:
            return self.bindings[name]
        elif self.parent:
            return self.parent.get(name)
        else:
            raise RuntimeError(f"Undefined variable: {name}")

    def set(self, name: str, value: TongValue) -> None:
        """Set a variable value"""
        if name in self.bindings:
            self.bindings[name] = value
        elif self.parent:
            self.parent.set(name, value)
        else:
            raise RuntimeError(f"Undefined variable: {name}")

class ParallelExecutor:
    """Executor for parallel operations"""

    def __init__(self, max_workers: Optional[int] = None):
        self.thread_pool = concurrent.futures.ThreadPoolExecutor(max_workers=max_workers)
        self.process_pool = concurrent.futures.ProcessPoolExecutor(max_workers=max_workers)

    def execute_parallel(self, tasks: List[Callable]) -> List[Any]:
        """Execute tasks in parallel using thread pool"""
        futures = [self.thread_pool.submit(task) for task in tasks]
        return [future.result() for future in concurrent.futures.as_completed(futures)]

    def execute_distributed(self, tasks: List[Callable]) -> List[Any]:
        """Execute tasks in distributed manner using process pool"""
        futures = [self.process_pool.submit(task) for task in tasks]
        return [future.result() for future in concurrent.futures.as_completed(futures)]

class GPUKernelExecutor:
    """Mock GPU kernel executor (would integrate with CUDA/OpenCL in real implementation)"""

    def __init__(self):
        self.available = False  # Set to True if GPU libraries are available

    def execute_kernel(self, kernel_func: Callable, args: List[Any], grid_size: int = 1, block_size: int = 1) -> Any:
        """Execute a GPU kernel function"""
        if not self.available:
            # Fallback to CPU execution
            return kernel_func(*args)

        # In real implementation, this would compile and execute on GPU
        # For now, we simulate with parallel CPU execution
        return kernel_func(*args)

class TongInterpreter:
    """TONG language interpreter with high-performance runtime"""

    def __init__(self):
        self.global_env = Environment()
        self.parallel_executor = ParallelExecutor()
        self.gpu_executor = GPUKernelExecutor()
        self.call_stack = []
        self._module_cache: Dict[str, TongModule] = {}
        self.setup_builtins()

    def setup_builtins(self) -> None:
        """Setup built-in functions and constants"""
        # Built-in functions
        self.global_env.define("print", TongFunction(self._builtin_print, "args..."))
        self.global_env.define("len", TongFunction(self._builtin_len, "array"))
        self.global_env.define("sum", TongFunction(self._builtin_sum, "array"))
        self.global_env.define("map", TongFunction(self._builtin_map, "array, func"))
        self.global_env.define("filter", TongFunction(self._builtin_filter, "array, func"))
        self.global_env.define("reduce", TongFunction(self._builtin_reduce, "array, func, initial"))
        self.global_env.define("import", TongFunction(self._builtin_import, "name"))

        # Mathematical constants
        self.global_env.define("PI", TongFloat(3.141592653589793))
        self.global_env.define("E", TongFloat(2.718281828459045))

    def _builtin_print(self, *args: TongValue) -> TongNone:
        """Built-in print function"""
        def format_value(val):
            if isinstance(val, TongArray):
                elements = [format_value(elem) for elem in val.value]
                return f"[{', '.join(elements)}]"
            else:
                return str(val.value)

        values = [format_value(arg) for arg in args]
        print(" ".join(values))
        return TongNone()

    def _builtin_len(self, arr: TongValue) -> TongInteger:
        """Built-in len function"""
        if isinstance(arr, TongArray):
            return TongInteger(len(arr.value))
        elif isinstance(arr, TongString):
            return TongInteger(len(arr.value))
        else:
            raise RuntimeError(f"len() not supported for type {arr.type_name}")

    def _builtin_sum(self, arr: TongValue) -> TongValue:
        """Built-in sum function with automatic parallelization"""
        if not isinstance(arr, TongArray):
            raise RuntimeError("sum() requires an array")

        elements = arr.value
        if not elements:
            return TongInteger(0)

        # Automatic parallelization for large arrays
        if len(elements) > 1000:
            return self._parallel_sum(elements)
        else:
            total = elements[0]
            for elem in elements[1:]:
                total = self._add_values(total, elem)
            return total

    def _parallel_sum(self, elements: List[TongValue]) -> TongValue:
        """Parallel sum implementation"""
        chunk_size = max(1, len(elements) // multiprocessing.cpu_count())
        chunks = [elements[i:i + chunk_size] for i in range(0, len(elements), chunk_size)]

        def sum_chunk(chunk):
            total = chunk[0]
            for elem in chunk[1:]:
                total = self._add_values(total, elem)
            return total

        chunk_sums = self.parallel_executor.execute_parallel([lambda ch=chunk: sum_chunk(ch) for chunk in chunks])

        result = chunk_sums[0]
        for chunk_sum in chunk_sums[1:]:
            result = self._add_values(result, chunk_sum)

        return result

    def _builtin_map(self, arr: TongValue, func: TongValue) -> TongArray:
        """Built-in map function with automatic parallelization"""
        if not isinstance(arr, TongArray):
            raise RuntimeError("map() requires an array")
        if not isinstance(func, TongFunction):
            raise RuntimeError("map() requires a function")

        elements = arr.value

        # Automatic parallelization for large arrays
        if len(elements) > 100:
            return self._parallel_map(elements, func)
        else:
            results = []
            for elem in elements:
                result = func.value(elem)
                results.append(result)
            return TongArray(results)

    def _parallel_map(self, elements: List[TongValue], func: TongFunction) -> TongArray:
        """Parallel map implementation"""
        def map_element(elem):
            return func.value(elem)

        results = self.parallel_executor.execute_parallel([lambda e=elem: map_element(e) for elem in elements])
        return TongArray(results)

    def _builtin_filter(self, arr: TongValue, func: TongValue) -> TongArray:
        """Built-in filter function"""
        if not isinstance(arr, TongArray):
            raise RuntimeError("filter() requires an array")
        if not isinstance(func, TongFunction):
            raise RuntimeError("filter() requires a function")

        results = []
        for elem in arr.value:
            if func.value(elem).value:  # Assumes function returns boolean
                results.append(elem)

        return TongArray(results)

    def _builtin_reduce(self, arr: TongValue, func: TongValue, initial: TongValue) -> TongValue:
        """Built-in reduce function"""
        if not isinstance(arr, TongArray):
            raise RuntimeError("reduce() requires an array")
        if not isinstance(func, TongFunction):
            raise RuntimeError("reduce() requires a function")

        accumulator = initial
        for elem in arr.value:
            accumulator = func.value(accumulator, elem)

        return accumulator

    # Module system -------------------------------------------------------
    def _builtin_import(self, name: TongValue) -> TongValue:
        """import(name: String) -> module

        Loads a module by name and returns a module value.
        """
        if not isinstance(name, TongString):
            raise RuntimeError("import() expects a string module name")
        modname = name.value
        if modname in self._module_cache:
            return self._module_cache[modname]
        module = self._load_module(modname)
        self._module_cache[modname] = module
        return module

    def _load_module(self, name: str) -> TongModule:
        """Load a built-in or external module by name."""
        if name == "sdl":
            return self._load_module_sdl()
        raise RuntimeError(f"Unknown module: {name}")

    # SDL module (ctypes-based SDL3 wrapper) -----------------------------
    def _load_module_sdl(self) -> TongModule:
        # Attempt to load the SDL3 shared library directly (no Python deps)
        def _load_sdl3() -> tuple[Any, str]:
            here = os.path.dirname(os.path.abspath(__file__))
            repo_root = os.path.abspath(os.path.join(here, os.pardir))
            if os.name == "nt":
                candidates = [
                    os.path.join(repo_root, "SDL3.dll"),
                    os.path.join(here, "SDL3.dll"),
                    "SDL3.dll",
                    "SDL3-0.dll",
                ]
            elif sys.platform == "darwin":
                candidates = [
                    os.path.join(repo_root, "libSDL3.dylib"),
                    os.path.join(here, "libSDL3.dylib"),
                    "libSDL3.dylib",
                    "SDL3.dylib",
                ]
            else:
                candidates = [
                    os.path.join(repo_root, "libSDL3.so"),
                    os.path.join(here, "libSDL3.so"),
                    "libSDL3.so",
                    "libSDL3-0.so",
                ]
            last_err: Optional[Exception] = None
            for n in candidates:
                try:
                    return CDLL(n), n
                except Exception as e:
                    last_err = e
            raise RuntimeError(
                "Unable to load SDL3 shared library. "
                f"Tried: {', '.join(candidates)}. "
                "Install the SDL3 runtime and ensure the library is on PATH (Windows), "
                "LD_LIBRARY_PATH (Linux), or DYLD_LIBRARY_PATH (macOS), or place it next to tong.py."
            ) from last_err

        sdl, _loaded_path = _load_sdl3()

        # Signatures
        sdl.SDL_Init.argtypes = [c_uint32]
        sdl.SDL_Init.restype = c_int
        sdl.SDL_Quit.argtypes = []
        sdl.SDL_Quit.restype = None
        try:
            sdl.SDL_SetMainReady.argtypes = []
            sdl.SDL_SetMainReady.restype = None
        except Exception:
            pass
        sdl.SDL_GetError.argtypes = []
        sdl.SDL_GetError.restype = c_char_p
        sdl.SDL_CreateWindow.argtypes = [c_char_p, c_int, c_int, c_uint32]
        sdl.SDL_CreateWindow.restype = c_void_p
        sdl.SDL_DestroyWindow.argtypes = [c_void_p]
        sdl.SDL_DestroyWindow.restype = None
        sdl.SDL_ShowWindow.argtypes = [c_void_p]
        sdl.SDL_ShowWindow.restype = None
        sdl.SDL_CreateRenderer.argtypes = [c_void_p, c_char_p]
        sdl.SDL_CreateRenderer.restype = c_void_p
        sdl.SDL_DestroyRenderer.argtypes = [c_void_p]
        sdl.SDL_DestroyRenderer.restype = None
        sdl.SDL_SetRenderDrawColor.argtypes = [c_void_p, c_uint8, c_uint8, c_uint8, c_uint8]
        sdl.SDL_SetRenderDrawColor.restype = c_int
        try:
            sdl.SDL_GetRenderDrawColor.argtypes = [c_void_p, POINTER(c_uint8), POINTER(c_uint8), POINTER(c_uint8), POINTER(c_uint8)]
            sdl.SDL_GetRenderDrawColor.restype = c_int
        except Exception:
            pass
        sdl.SDL_RenderClear.argtypes = [c_void_p]
        sdl.SDL_RenderClear.restype = c_int
        sdl.SDL_RenderPresent.argtypes = [c_void_p]
        sdl.SDL_RenderPresent.restype = None
        try:
            sdl.SDL_RenderSetScale.argtypes = [c_void_p, c_float, c_float]
            sdl.SDL_RenderSetScale.restype = c_int
        except Exception:
            pass
        # Optional APIs
        has_set_viewport = False
        try:
            sdl.SDL_RenderSetViewport.restype = c_int
            has_set_viewport = True
        except Exception:
            try:
                sdl.SDL_SetRenderViewport.restype = c_int
                has_set_viewport = True
            except Exception:
                pass
        has_output_size = False
        try:
            sdl.SDL_GetRenderOutputSize.argtypes = [c_void_p, POINTER(c_int), POINTER(c_int)]
            sdl.SDL_GetRenderOutputSize.restype = c_int
            has_output_size = True
        except Exception:
            try:
                sdl.SDL_GetRendererOutputSize.argtypes = [c_void_p, POINTER(c_int), POINTER(c_int)]
                sdl.SDL_GetRendererOutputSize.restype = c_int
                has_output_size = True
            except Exception:
                pass
        has_window_size = False
        try:
            sdl.SDL_GetWindowSize.argtypes = [c_void_p, POINTER(c_int), POINTER(c_int)]
            sdl.SDL_GetWindowSize.restype = None
            has_window_size = True
        except Exception:
            pass
        # Rects
        class SDL_FRect(Structure):
            _fields_ = [("x", c_float), ("y", c_float), ("w", c_float), ("h", c_float)]
        class SDL_Rect(Structure):
            _fields_ = [("x", c_int), ("y", c_int), ("w", c_int), ("h", c_int)]
        use_frect = False
        try:
            sdl.SDL_RenderFillRectF.argtypes = [c_void_p, POINTER(SDL_FRect)]
            sdl.SDL_RenderFillRectF.restype = c_int
            use_frect = True
        except Exception:
            pass
        try:
            sdl.SDL_RenderFillRect.argtypes = [c_void_p, POINTER(SDL_Rect)]
            sdl.SDL_RenderFillRect.restype = c_int
        except Exception:
            pass
        try:
            sdl.SDL_RenderSetViewport.argtypes = [c_void_p, POINTER(SDL_Rect)]
        except Exception:
            try:
                sdl.SDL_SetRenderViewport.argtypes = [c_void_p, POINTER(SDL_Rect)]
            except Exception:
                pass
        sdl.SDL_Delay.argtypes = [c_uint32]
        sdl.SDL_Delay.restype = None
        class SDL_Event(Structure):
            _fields_ = [("type", c_uint32), ("_pad", c_uint8 * 200)]
        sdl.SDL_PollEvent.argtypes = [POINTER(SDL_Event)]
        sdl.SDL_PollEvent.restype = c_int
        try:
            sdl.SDL_PumpEvents.argtypes = []
            sdl.SDL_PumpEvents.restype = None
        except Exception:
            pass
        sdl.SDL_GetKeyboardState.argtypes = [POINTER(c_int)]
        sdl.SDL_GetKeyboardState.restype = POINTER(c_uint8)
        try:
            sdl.SDL_SetHint.argtypes = [c_char_p, c_char_p]
            sdl.SDL_SetHint.restype = c_int
        except Exception:
            pass
        has_quit_requested = False
        try:
            sdl.SDL_QuitRequested.argtypes = []
            sdl.SDL_QuitRequested.restype = c_int
            has_quit_requested = True
        except Exception:
            pass

        # Constants
        SDL_INIT_VIDEO = 0x00000020
        SDL_INIT_EVENTS = 0x00004000
        SDL_WINDOW_RESIZABLE = 0x00000020
        SDL_WINDOW_SHOWN = 0x00000004
        SDL_EVENT_QUIT = 0x100
        SDL_SCANCODE_UP = 82
        SDL_SCANCODE_DOWN = 81
        SDL_SCANCODE_W = 26
        SDL_SCANCODE_S = 22
        SDL_SCANCODE_ESCAPE = 41
        SDL_SCANCODE_Q = 20

        exports: Dict[str, TongValue] = {}

        # Helpers
        def _to_int(v: TongValue) -> int:
            if isinstance(v, TongInteger):
                return v.value
            raise RuntimeError("Expected integer")
        def _to_str(v: TongValue) -> str:
            if isinstance(v, TongString):
                return v.value
            raise RuntimeError("Expected string")

        def _sdl_init() -> TongValue:
            # Clear any conflicting environment variables first
            if "SDL_VIDEODRIVER" in os.environ:
                del os.environ["SDL_VIDEODRIVER"]
            if "SDL_RENDER_DRIVER" in os.environ:
                del os.environ["SDL_RENDER_DRIVER"]

            try:
                if hasattr(sdl, "SDL_SetHint"):
                    # Don't force software rendering by default
                    # sdl.SDL_SetHint(b"SDL_RENDER_DRIVER", b"software")
                    try:
                        sdl.SDL_SetHint(b"SDL_QUIT_ON_LAST_WINDOW_CLOSE", b"1")
                    except Exception:
                        pass
            except Exception:
                pass

            # On Windows, let SDL choose the best video driver automatically
            # if os.name == "nt" and not os.environ.get("SDL_VIDEODRIVER"):
            #     os.environ["SDL_VIDEODRIVER"] = "windows"

            try:
                if hasattr(sdl, "SDL_SetMainReady"):
                    sdl.SDL_SetMainReady()
            except Exception:
                pass
            init_flags = [0, SDL_INIT_VIDEO, SDL_INIT_VIDEO | SDL_INIT_EVENTS]
            last_err_msg = None
            for fl in init_flags:
                res = sdl.SDL_Init(c_uint32(fl))
                if not res:
                    last_err_msg = None
                    break
                err_ptr = sdl.SDL_GetError()
                last_err_msg = err_ptr.decode("utf-8", errors="ignore") if err_ptr else None
            if last_err_msg is not None:
                try:
                    os.environ["SDL_VIDEODRIVER"] = "dummy"
                    res = sdl.SDL_Init(c_uint32(0))
                    if res == 0:
                        last_err_msg = None
                except Exception:
                    pass
            if last_err_msg is not None:
                raise RuntimeError(f"SDL_Init failed: {last_err_msg or '(no error message)'} (library: {_loaded_path})")
            return TongNone()

        def _sdl_quit() -> TongValue:
            sdl.SDL_Quit()
            return TongNone()

        def _sdl_create_window(title: TongValue, w: TongValue, h: TongValue) -> TongExternal:
            c_title = _to_str(title).encode("utf-8")
            win = sdl.SDL_CreateWindow(c_title, c_int(_to_int(w)), c_int(_to_int(h)), c_uint32(SDL_WINDOW_SHOWN | SDL_WINDOW_RESIZABLE))
            if not win:
                err_ptr = sdl.SDL_GetError()
                err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                raise RuntimeError(f"SDL_CreateWindow failed: {err}")
            return TongExternal(win, "SDL_Window")

        def _sdl_destroy_window(win: TongExternal) -> TongValue:
            sdl.SDL_DestroyWindow(win.value)
            return TongNone()

        def _sdl_create_renderer(win: TongExternal) -> TongExternal:
            renderer = None
            drivers_to_try = [None, b"opengl", b"software", b"direct3d", b"opengles2", b"metal"]

            for driver_name in drivers_to_try:
                try:
                    renderer = sdl.SDL_CreateRenderer(win.value, driver_name)
                    if renderer:
                        print(f"SDL: Using renderer driver: {driver_name.decode() if driver_name else 'default'}")
                        break
                except Exception as e:
                    print(f"SDL: Failed to create renderer with {driver_name}: {e}")
                    renderer = None

            if not renderer:
                if os.environ.get("TONG_SDL_HEADLESS") == "1":
                    return TongExternal("noop", "SDL_Renderer")
                err_ptr = sdl.SDL_GetError()
                err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                raise RuntimeError(f"SDL_CreateRenderer failed with all drivers: {err}")
            try:
                if hasattr(sdl, 'SDL_GetRenderOutputSize') and hasattr(sdl, 'SDL_RenderSetScale'):
                    w = c_int(0); h = c_int(0)
                    if sdl.SDL_GetRenderOutputSize(renderer, byref(w), byref(h)) == 0 and (w.value == 0 or h.value == 0):
                        sdl.SDL_RenderSetScale(renderer, c_float(1.0), c_float(1.0))
                elif hasattr(sdl, 'SDL_GetRendererOutputSize') and hasattr(sdl, 'SDL_RenderSetScale'):
                    w = c_int(0); h = c_int(0)
                    if sdl.SDL_GetRendererOutputSize(renderer, byref(w), byref(h)) == 0 and (w.value == 0 or h.value == 0):
                        sdl.SDL_RenderSetScale(renderer, c_float(1.0), c_float(1.0))
                if has_window_size and has_set_viewport:
                    ww = c_int(0); hh = c_int(0)
                    try:
                        sdl.SDL_GetWindowSize(win.value, byref(ww), byref(hh))
                        if ww.value > 0 and hh.value > 0:
                            vr = SDL_Rect(c_int(0), c_int(0), c_int(ww.value), c_int(hh.value))
                            try:
                                if hasattr(sdl, 'SDL_RenderSetViewport'):
                                    sdl.SDL_RenderSetViewport(renderer, byref(vr))
                                elif hasattr(sdl, 'SDL_SetRenderViewport'):
                                    sdl.SDL_SetRenderViewport(renderer, byref(vr))
                            except Exception:
                                pass
                    except Exception:
                        pass
            except Exception:
                pass
            return TongExternal(renderer, "SDL_Renderer")

        def _sdl_destroy_renderer(renderer: TongExternal) -> TongValue:
            if renderer.value != "noop":
                sdl.SDL_DestroyRenderer(renderer.value)
            return TongNone()

        def _sdl_set_draw_color(renderer: TongExternal, r: TongValue, g: TongValue, b: TongValue, a: TongValue) -> TongValue:
            if renderer.value != "noop":
                rc = sdl.SDL_SetRenderDrawColor(renderer.value, c_uint8(_to_int(r)), c_uint8(_to_int(g)), c_uint8(_to_int(b)), c_uint8(_to_int(a)))
                if rc < 0:
                    err_ptr = sdl.SDL_GetError()
                    err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                    raise RuntimeError(f"SDL_SetRenderDrawColor failed: {err}")
            return TongNone()

        def _sdl_clear(renderer: TongExternal) -> TongValue:
            if renderer.value != "noop":
                rc = sdl.SDL_RenderClear(renderer.value)
                if rc < 0:
                    err_ptr = sdl.SDL_GetError()
                    err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                    raise RuntimeError(f"SDL_RenderClear failed: {err}")
            return TongNone()

        def _sdl_present(renderer: TongExternal) -> TongValue:
            if renderer.value != "noop":
                sdl.SDL_RenderPresent(renderer.value)
            return TongNone()

        def _sdl_set_scale(renderer: TongExternal, sx: TongValue, sy: TongValue) -> TongValue:
            if renderer.value != "noop" and hasattr(sdl, 'SDL_RenderSetScale'):
                rc = sdl.SDL_RenderSetScale(renderer.value, c_float(float(_to_int(sx))), c_float(float(_to_int(sy))))
                if rc < 0:
                    err_ptr = sdl.SDL_GetError()
                    err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                    raise RuntimeError(f"SDL_RenderSetScale failed: {err}")
            return TongNone()

        def _sdl_output_size(renderer: TongExternal) -> TongArray:
            if renderer.value == "noop":
                return TongArray([TongInteger(0), TongInteger(0)])
            w = c_int(0)
            h = c_int(0)
            rc = -1
            try:
                if hasattr(sdl, 'SDL_GetRenderOutputSize'):
                    rc = sdl.SDL_GetRenderOutputSize(renderer.value, byref(w), byref(h))
                elif hasattr(sdl, 'SDL_GetRendererOutputSize'):
                    rc = sdl.SDL_GetRendererOutputSize(renderer.value, byref(w), byref(h))
            except Exception:
                rc = -1
            if rc < 0:
                return TongArray([TongInteger(0), TongInteger(0)])
            return TongArray([TongInteger(w.value), TongInteger(h.value)])

        def _sdl_window_size(win: TongExternal) -> TongArray:
            if not has_window_size:
                return TongArray([TongInteger(0), TongInteger(0)])
            w = c_int(0); h = c_int(0)
            try:
                sdl.SDL_GetWindowSize(win.value, byref(w), byref(h))
            except Exception:
                pass
            return TongArray([TongInteger(w.value), TongInteger(h.value)])

        def _sdl_set_viewport(renderer: TongExternal, x: TongValue, y: TongValue, w: TongValue, h: TongValue) -> TongValue:
            if renderer.value == "noop":
                return TongNone()
            if not has_set_viewport:
                return TongNone()
            rect = SDL_Rect(c_int(_to_int(x)), c_int(_to_int(y)), c_int(_to_int(w)), c_int(_to_int(h)))
            try:
                if hasattr(sdl, 'SDL_RenderSetViewport'):
                    rc = sdl.SDL_RenderSetViewport(renderer.value, byref(rect))
                else:
                    rc = sdl.SDL_SetRenderViewport(renderer.value, byref(rect))
                if rc < 0:
                    err_ptr = sdl.SDL_GetError()
                    err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                    raise RuntimeError(f"SDL_RenderSetViewport failed: {err}")
            except Exception:
                pass
            return TongNone()

        def _sdl_fill_rect(renderer: TongExternal, x: TongValue, y: TongValue, w: TongValue, h: TongValue, r: TongValue, g: TongValue, b: TongValue, a: TongValue) -> TongValue:
            if renderer.value == "noop":
                return TongNone()

            # Set the draw color for this rectangle
            rc = sdl.SDL_SetRenderDrawColor(renderer.value, c_uint8(_to_int(r)), c_uint8(_to_int(g)), c_uint8(_to_int(b)), c_uint8(_to_int(a)))
            if rc < 0:
                err_ptr = sdl.SDL_GetError()
                err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                print(f"SDL_SetRenderDrawColor failed: {err}")
                return TongNone()

            # Create rectangle
            rect = SDL_Rect(c_int(_to_int(x)), c_int(_to_int(y)), c_int(_to_int(w)), c_int(_to_int(h)))
            print(f"Drawing rect at ({_to_int(x)}, {_to_int(y)}) size ({_to_int(w)}, {_to_int(h)}) color ({_to_int(r)}, {_to_int(g)}, {_to_int(b)}, {_to_int(a)})")

            # Draw the rectangle
            rc = sdl.SDL_RenderFillRect(renderer.value, byref(rect))
            if rc < 0:
                err_ptr = sdl.SDL_GetError()
                err = err_ptr.decode("utf-8", errors="ignore") if err_ptr else "(no error message)"
                print(f"SDL_RenderFillRect failed: {err}")
                return TongNone()

            print(f"Successfully drew rect, rc={rc}")
            return TongNone()

        def _sdl_delay(ms: TongValue) -> TongValue:
            sdl.SDL_Delay(c_uint32(_to_int(ms)))
            return TongNone()

        def _sdl_poll_quit() -> TongBoolean:
            ev = SDL_Event()
            try:
                if has_quit_requested and sdl.SDL_QuitRequested() != 0:
                    return TongBoolean(True)
            except Exception:
                pass
            while sdl.SDL_PollEvent(byref(ev)):
                if ev.type == SDL_EVENT_QUIT:
                    return TongBoolean(True)
            return TongBoolean(False)

        def _sdl_key_down(keycode: TongValue) -> TongBoolean:
            try:
                if hasattr(sdl, 'SDL_PumpEvents'):
                    sdl.SDL_PumpEvents()
            except Exception:
                pass
            num = c_int(0)
            state = sdl.SDL_GetKeyboardState(byref(num))
            kc = _to_int(keycode)
            if not state or kc < 0 or kc >= num.value:
                return TongBoolean(False)
            return TongBoolean(state[kc] != 0)

        # Exports
        exports["K_UP"] = TongInteger(SDL_SCANCODE_UP)
        exports["K_DOWN"] = TongInteger(SDL_SCANCODE_DOWN)
        exports["K_W"] = TongInteger(SDL_SCANCODE_W)
        exports["K_S"] = TongInteger(SDL_SCANCODE_S)
        exports["K_ESCAPE"] = TongInteger(SDL_SCANCODE_ESCAPE)
        exports["K_Q"] = TongInteger(SDL_SCANCODE_Q)

        exports["init"] = TongFunction(_sdl_init, "")
        exports["quit"] = TongFunction(_sdl_quit, "")
        exports["create_window"] = TongFunction(
            lambda title, w, h: _sdl_create_window(title, w, h), "title, w, h")
        exports["destroy_window"] = TongFunction(
            lambda win: _sdl_destroy_window(win), "win")
        exports["create_renderer"] = TongFunction(
            lambda win: _sdl_create_renderer(win), "win")
        exports["destroy_renderer"] = TongFunction(
            lambda r: _sdl_destroy_renderer(r), "renderer")
        exports["set_draw_color"] = TongFunction(
            lambda r, rr, gg, bb, aa: _sdl_set_draw_color(r, rr, gg, bb, aa),
            "renderer, r, g, b, a")
        exports["clear"] = TongFunction(lambda r: _sdl_clear(r), "renderer")
        exports["present"] = TongFunction(lambda r: _sdl_present(r), "renderer")
        exports["fill_rect"] = TongFunction(
            lambda r, x, y, w, h, rr, gg, bb, aa: _sdl_fill_rect(
                r, x, y, w, h, rr, gg, bb, aa),
            "renderer, x, y, w, h, r, g, b, a")
        exports["delay"] = TongFunction(lambda ms: _sdl_delay(ms), "ms")
        exports["poll_quit"] = TongFunction(_sdl_poll_quit, "")
        exports["key_down"] = TongFunction(
            lambda kc: _sdl_key_down(kc), "keycode")
        exports["set_scale"] = TongFunction(
            lambda r, sx, sy: _sdl_set_scale(r, sx, sy), "renderer, sx, sy")
        exports["output_size"] = TongFunction(
            lambda r: _sdl_output_size(r), "renderer")
        exports["window_size"] = TongFunction(
            lambda w: _sdl_window_size(w), "window")
        exports["set_viewport"] = TongFunction(
            lambda r, x, y, w, h: _sdl_set_viewport(r, x, y, w, h),
            "renderer, x, y, w, h")

        # Diagnostics
        try:
            sdl.SDL_GetCurrentVideoDriver.argtypes = []
            sdl.SDL_GetCurrentVideoDriver.restype = c_char_p
            exports["current_driver"] = TongFunction(lambda: TongString((sdl.SDL_GetCurrentVideoDriver() or b"").decode("utf-8", errors="ignore")), "")
        except Exception:
            exports["current_driver"] = TongFunction(lambda: TongString("unknown"), "")
        try:
            sdl.SDL_GetRendererName.argtypes = [c_void_p]
            sdl.SDL_GetRendererName.restype = c_char_p
            exports["renderer_name"] = TongFunction(lambda r: TongString((sdl.SDL_GetRendererName(r.value) or b"").decode("utf-8", errors="ignore")) if isinstance(r, TongExternal) and r.type_name=="SDL_Renderer" and r.value!="noop" else TongString("noop"), "renderer")
        except Exception:
            exports["renderer_name"] = TongFunction(lambda r: TongString("unknown"), "renderer")
        exports["loaded_path"] = TongFunction(lambda: TongString(_loaded_path), "")

        return TongModule("sdl", exports)

    def interpret(self, program: Program) -> None:
        """Interpret a TONG program"""
        try:
            for statement in program.statements:
                self.execute_statement(statement, self.global_env)
        except Exception as e:
            print(f"Runtime error: {e}")
            raise

    def execute_statement(self, stmt: Statement, env: Environment) -> Any:
        """Execute a statement"""
        if isinstance(stmt, ExpressionStatement):
            return self.evaluate_expression(stmt.expression, env)

        elif isinstance(stmt, VariableDeclaration):
            value = TongNone()
            if stmt.initializer:
                value = self.evaluate_expression(stmt.initializer, env)
            env.define(stmt.name, value)
            return None

        elif isinstance(stmt, Assignment):
            value = self.evaluate_expression(stmt.value, env)
            if isinstance(stmt.target, Identifier):
                env.set(stmt.target.name, value)
            else:
                raise RuntimeError("Invalid assignment target")
            return None

        elif isinstance(stmt, FunctionDeclaration):
            func = self._create_function(stmt, env)
            env.define(stmt.name, func)
            return None

        elif isinstance(stmt, ReturnStatement):
            if stmt.value:
                return self.evaluate_expression(stmt.value, env)
            return TongNone()

        elif isinstance(stmt, IfStatement):
            condition = self.evaluate_expression(stmt.condition, env)
            if self._is_truthy(condition):
                for s in stmt.then_body:
                    result = self.execute_statement(s, env)
                    if isinstance(s, ReturnStatement):
                        return result
            elif stmt.else_body:
                for s in stmt.else_body:
                    result = self.execute_statement(s, env)
                    if isinstance(s, ReturnStatement):
                        return result
            return None

        elif isinstance(stmt, WhileLoop):
            while True:
                condition = self.evaluate_expression(stmt.condition, env)
                if not self._is_truthy(condition):
                    break

                for s in stmt.body:
                    result = self.execute_statement(s, env)
                    if isinstance(s, (ReturnStatement, BreakStatement)):
                        return result
                    elif isinstance(s, ContinueStatement):
                        break
            return None

        elif isinstance(stmt, (BreakStatement, ContinueStatement)):
            return stmt  # Let parent handle control flow

        else:
            raise RuntimeError(f"Unknown statement type: {type(stmt)}")

    def evaluate_expression(self, expr: Expression, env: Environment) -> TongValue:
        """Evaluate an expression"""
        if isinstance(expr, IntegerLiteral):
            return TongInteger(expr.value)

        elif isinstance(expr, FloatLiteral):
            return TongFloat(expr.value)

        elif isinstance(expr, StringLiteral):
            return TongString(expr.value)

        elif isinstance(expr, BooleanLiteral):
            return TongBoolean(expr.value)

        elif isinstance(expr, NoneLiteral):
            return TongNone()

        elif isinstance(expr, Identifier):
            return env.get(expr.name)

        elif isinstance(expr, ArrayLiteral):
            elements = [self.evaluate_expression(elem, env) for elem in expr.elements]
            return TongArray(elements)

        elif isinstance(expr, BinaryOperation):
            left = self.evaluate_expression(expr.left, env)
            right = self.evaluate_expression(expr.right, env)
            return self._evaluate_binary_op(left, expr.operator, right)

        elif isinstance(expr, UnaryOperation):
            operand = self.evaluate_expression(expr.operand, env)
            return self._evaluate_unary_op(expr.operator, operand)

        elif isinstance(expr, FunctionCall):
            func = self.evaluate_expression(expr.function, env)
            args = [self.evaluate_expression(arg, env) for arg in expr.arguments]
            return self._call_function(func, args)

        elif isinstance(expr, MethodCall):
            # Evaluate receiver, then fetch field/method and call
            obj = self.evaluate_expression(expr.object, env)
            method = self._get_field(obj, expr.method)
            args = [self.evaluate_expression(arg, env) for arg in expr.arguments]
            return self._call_function(method, args)

        elif isinstance(expr, FieldAccess):
            obj = self.evaluate_expression(expr.object, env)
            return self._get_field(obj, expr.field)

        elif isinstance(expr, IndexAccess):
            obj = self.evaluate_expression(expr.object, env)
            index = self.evaluate_expression(expr.index, env)
            return self._get_index(obj, index)

        elif isinstance(expr, ParallelBlock):
            return self._execute_parallel_block(expr, env)

        elif isinstance(expr, DistributedBlock):
            return self._execute_distributed_block(expr, env)

        elif isinstance(expr, AwaitExpression):
            return self._execute_await(expr, env)

        elif isinstance(expr, Lambda):
            return self._create_lambda(expr, env)

        else:
            raise RuntimeError(f"Unknown expression type: {type(expr)}")

    def _create_function(self, func_decl: FunctionDeclaration, closure_env: Environment) -> TongFunction:
        """Create a function value from declaration"""
        def tong_function(*args: TongValue) -> TongValue:
            # Create new environment for function execution
            func_env = Environment(closure_env)

            # Bind parameters
            for i, param in enumerate(func_decl.parameters):
                if i < len(args):
                    func_env.define(param.name, args[i])
                elif param.default_value:
                    default = self.evaluate_expression(param.default_value, closure_env)
                    func_env.define(param.name, default)
                else:
                    raise RuntimeError(f"Missing argument for parameter {param.name}")

            # Execute function body
            for stmt in func_decl.body:
                result = self.execute_statement(stmt, func_env)
                if isinstance(stmt, ReturnStatement):
                    return result if result else TongNone()

            return TongNone()

        return TongFunction(tong_function, f"{func_decl.name}(...)")

    def _call_function(self, func: TongValue, args: List[TongValue]) -> TongValue:
        """Call a function"""
        if not isinstance(func, TongFunction):
            raise RuntimeError(f"Cannot call non-function value: {func.type_name}")

        return func.value(*args)

    def _create_lambda(self, lam: Lambda, closure_env: Environment) -> TongFunction:
        """Create a function value from a lambda expression, capturing the environment"""
        def lambda_func(*args: TongValue) -> TongValue:
            func_env = Environment(closure_env)
            # Bind parameters by position
            for i, param in enumerate(lam.parameters):
                if i < len(args):
                    func_env.define(param.name, args[i])
                else:
                    # No defaults for lambdas in current spec
                    raise RuntimeError(f"Missing argument for parameter {param.name}")
            # Evaluate lambda body expression
            return self.evaluate_expression(lam.body, func_env)

        sig = ", ".join(p.name for p in lam.parameters)
        return TongFunction(lambda_func, f"lambda({sig})")

    def _execute_parallel_block(self, block: ParallelBlock, env: Environment) -> TongValue:
        """Execute a parallel block"""
        # For simplicity, execute statements in parallel using thread pool
        def execute_stmt(stmt):
            return self.execute_statement(stmt, env)

        tasks = [lambda s=stmt: execute_stmt(s) for stmt in block.statements]
        results = self.parallel_executor.execute_parallel(tasks)

        # Return the last non-None result
        for result in reversed(results):
            if result is not None:
                return result

        return TongNone()

    def _execute_distributed_block(self, block: DistributedBlock, env: Environment) -> TongValue:
        """Execute a distributed block"""
        # Similar to parallel but uses process pool
        def execute_stmt(stmt):
            return self.execute_statement(stmt, env)

        tasks = [
            lambda s=stmt: execute_stmt(s) for stmt in block.statements
        ]
        results = self.parallel_executor.execute_distributed(tasks)

        for result in reversed(results):
            if result is not None:
                return result

        return TongNone()

    def _execute_await(self, await_expr: AwaitExpression, env: Environment) -> TongValue:
        """Execute await expression"""
        # Simplified async execution
        result = self.evaluate_expression(await_expr.expr, env)
        return result

    def _evaluate_binary_op(self, left: TongValue, op: BinaryOperator, right: TongValue) -> TongValue:
        """Evaluate binary operation"""
        if op == BinaryOperator.ADD:
            return self._add_values(left, right)
        elif op == BinaryOperator.SUBTRACT:
            return self._subtract_values(left, right)
        elif op == BinaryOperator.MULTIPLY:
            return self._multiply_values(left, right)
        elif op == BinaryOperator.DIVIDE:
            return self._divide_values(left, right)
        elif op == BinaryOperator.MODULO:
            return self._modulo_values(left, right)
        elif op == BinaryOperator.EQUAL:
            return TongBoolean(left.value == right.value)
        elif op == BinaryOperator.NOT_EQUAL:
            return TongBoolean(left.value != right.value)
        elif op == BinaryOperator.LESS_THAN:
            return TongBoolean(left.value < right.value)
        elif op == BinaryOperator.LESS_EQUAL:
            return TongBoolean(left.value <= right.value)
        elif op == BinaryOperator.GREATER_THAN:
            return TongBoolean(left.value > right.value)
        elif op == BinaryOperator.GREATER_EQUAL:
            return TongBoolean(left.value >= right.value)
        elif op == BinaryOperator.AND:
            return TongBoolean(self._is_truthy(left) and self._is_truthy(right))
        elif op == BinaryOperator.OR:
            return TongBoolean(self._is_truthy(left) or self._is_truthy(right))
        else:
            raise RuntimeError(f"Unknown binary operator: {op}")

    def _evaluate_unary_op(self, op: UnaryOperator, operand: TongValue) -> TongValue:
        """Evaluate unary operation"""
        if op == UnaryOperator.NEGATE:
            if isinstance(operand, TongInteger):
                return TongInteger(-operand.value)
            elif isinstance(operand, TongFloat):
                return TongFloat(-operand.value)
            else:
                raise RuntimeError(f"Cannot negate {operand.type_name}")
        elif op == UnaryOperator.NOT:
            return TongBoolean(not self._is_truthy(operand))
        else:
            raise RuntimeError(f"Unknown unary operator: {op}")

    def _add_values(self, left: TongValue, right: TongValue) -> TongValue:
        """Add two values"""
        if isinstance(left, TongInteger) and isinstance(right, TongInteger):
            return TongInteger(left.value + right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongFloat):
            return TongFloat(left.value + right.value)
        elif isinstance(left, TongInteger) and isinstance(right, TongFloat):
            return TongFloat(left.value + right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongInteger):
            return TongFloat(left.value + right.value)
        elif isinstance(left, TongString) and isinstance(right, TongString):
            return TongString(left.value + right.value)
        else:
            raise RuntimeError(f"Cannot add {left.type_name} and {right.type_name}")

    def _subtract_values(self, left: TongValue, right: TongValue) -> TongValue:
        """Subtract two values"""
        if isinstance(left, TongInteger) and isinstance(right, TongInteger):
            return TongInteger(left.value - right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongFloat):
            return TongFloat(left.value - right.value)
        elif isinstance(left, TongInteger) and isinstance(right, TongFloat):
            return TongFloat(left.value - right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongInteger):
            return TongFloat(left.value - right.value)
        else:
            raise RuntimeError(f"Cannot subtract {left.type_name} and {right.type_name}")

    def _multiply_values(self, left: TongValue, right: TongValue) -> TongValue:
        """Multiply two values"""
        if isinstance(left, TongInteger) and isinstance(right, TongInteger):
            return TongInteger(left.value * right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongFloat):
            return TongFloat(left.value * right.value)
        elif isinstance(left, TongInteger) and isinstance(right, TongFloat):
            return TongFloat(left.value * right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongInteger):
            return TongFloat(left.value * right.value)
        else:
            raise RuntimeError(f"Cannot multiply {left.type_name} and {right.type_name}")

    def _divide_values(self, left: TongValue, right: TongValue) -> TongValue:
        """Divide two values"""
        if isinstance(right, (TongInteger, TongFloat)) and right.value == 0:
            raise RuntimeError("Division by zero")

        if isinstance(left, TongInteger) and isinstance(right, TongInteger):
            return TongFloat(left.value / right.value)  # Always return float for division
        elif isinstance(left, TongFloat) and isinstance(right, TongFloat):
            return TongFloat(left.value / right.value)
        elif isinstance(left, TongInteger) and isinstance(right, TongFloat):
            return TongFloat(left.value / right.value)
        elif isinstance(left, TongFloat) and isinstance(right, TongInteger):
            return TongFloat(left.value / right.value)
        else:
            raise RuntimeError(f"Cannot divide {left.type_name} and {right.type_name}")

    def _modulo_values(self, left: TongValue, right: TongValue) -> TongValue:
        """Modulo operation"""
        if isinstance(left, TongInteger) and isinstance(right, TongInteger):
            return TongInteger(left.value % right.value)
        else:
            raise RuntimeError(f"Cannot perform modulo on {left.type_name} and {right.type_name}")

    def _get_field(self, obj: TongValue, field: str) -> TongValue:
        """Get field from object"""
        if isinstance(obj, TongArray) and field == "length":
            return TongInteger(len(obj.value))
        elif isinstance(obj, TongString) and field == "length":
            return TongInteger(len(obj.value))
        elif isinstance(obj, TongModule):
            if field in obj.exports:
                return obj.exports[field]
            raise RuntimeError(f"Module '{obj.name}' has no export '{field}'")
        else:
            raise RuntimeError(f"No field '{field}' on type {obj.type_name}")

    def _get_index(self, obj: TongValue, index: TongValue) -> TongValue:
        """Get index from object"""
        if isinstance(obj, TongArray) and isinstance(index, TongInteger):
            if 0 <= index.value < len(obj.value):
                return obj.value[index.value]
            else:
                raise RuntimeError("Array index out of bounds")
        elif isinstance(obj, TongString) and isinstance(index, TongInteger):
            if 0 <= index.value < len(obj.value):
                return TongString(obj.value[index.value])
            else:
                raise RuntimeError("String index out of bounds")
        else:
            raise RuntimeError(f"Cannot index {obj.type_name} with {index.type_name}")

    def _is_truthy(self, value: TongValue) -> bool:
        """Check if value is truthy"""
        if isinstance(value, TongBoolean):
            return value.value
        elif isinstance(value, TongNone):
            return False
        elif isinstance(value, TongInteger):
            return value.value != 0
        elif isinstance(value, TongFloat):
            return value.value != 0.0
        elif isinstance(value, TongString):
            return len(value.value) > 0
        elif isinstance(value, TongArray):
            return len(value.value) > 0
        else:
            return True
