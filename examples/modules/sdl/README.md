# SDL Pong example (ctypes-based SDL3)

This example uses Tong's native module system to import an `sdl` module that binds directly to the SDL3 runtime via ctypes. No Python SDL package is required.

Prerequisites
- Install the SDL3 runtime/shared library and ensure it can be found by the loader:
  - Windows: copy `SDL3.dll` (or `SDL3-0.dll`) to a folder on your PATH or next to `tong.py`.
  - macOS: install `libSDL3.dylib` and ensure it's discoverable (e.g., set `DYLD_LIBRARY_PATH`).
  - Linux: install `libSDL3.so` and ensure it's discoverable (e.g., set `LD_LIBRARY_PATH`).

Run
```
python tong.py examples/modules/sdl/pong.tong
```

Notes
- The automated examples runner skips this folder because it requires a window and user input.
- If you see "Unable to load SDL3 shared library", add the SDL3 library to PATH or place it next to `tong.py`.
- If SDL initialization fails, the interpreter will print the SDL error message.