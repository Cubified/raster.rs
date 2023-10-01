# raster.rs

A terminal-based, interactive 3D software renderer written in Rust.

Capable of parsing and displaying OBJ 3D models with TGA textures and normal maps, directly in any modern terminal emulator.

Calculations are parallelized with [rayon](https://github.com/rayon-rs/rayon) for optimized multi-core performance.

## Screenshots and Demos

![demo.gif](https://github.com/Cubified/raster.rs/blob/main/img/demo.gif)

![pill.png](https://github.com/Cubified/raster.rs/blob/main/img/pill.png)

![head.png](https://github.com/Cubified/raster.rs/blob/main/img/head.png)

![teapot.png](https://github.com/Cubified/raster.rs/blob/main/img/teapot.png)

## Building and Running

First, ensure an up-to-date Rust toolchain is installed (preferably from [rustup.rs](https://rustup.rs)).

Then, run:

```sh
$ cargo run obj/head.obj map/head_diffuse.tga
```

The only required argument is the model `.obj` file, but diffuse, normal, and specular maps can be specified:

```sh
$ cargo run obj/head.obj map/head_diffuse.tga map/head_normal.tga map/head_specular.tga
```

## Per-File Technical Overview

- [`UI`](https://github.com/Cubified/raster.rs/blob/main/src/ui.rs#L22):  Escape sequence handler for mouse inputs.  Opens a channel on a second thread to read from stdin without blocking the render thread.
- [`EscSeq`](https://github.com/Cubified/raster.rs/blob/main/src/esc.rs#L12):  An escape sequence parser using a basic DFA/state machine.  Somewhat inspired by [`vt100utils`](https://github.com/Cubified/vt100utils).
- [`Model`](https://github.com/Cubified/raster.rs/blob/main/src/model.rs#L19):  OBJ file parser.  Builds vertex, normal, and UV buffers from valid OBJ commands.
- [`Shader`](https://github.com/Cubified/raster.rs/blob/main/src/shader.rs#L19):  Vertex and fragment shaders, plus the rendering routine responsible for displaying an entire frame of pixels.  Uses some basic string comparisons as optimizations, because writing unnecessary graphics commands to stdout would be significantly slower.
- [`Orbit`](https://github.com/Cubified/raster.rs/blob/main/src/orbit.rs#L12):  3D orbit controls.  Largely ported from Three.js' [OrbitControls](https://github.com/mrdoob/three.js/blob/309e5f6f64c7af9087e0fb6f7cbf83a9fd2a4fef/examples/jsm/controls/OrbitControls.js).
- [`Vertex`](https://github.com/Cubified/raster.rs/blob/main/src/vertex.rs#L10):  Vertex math, including [barycentric coordinates](https://en.wikipedia.org/wiki/Barycentric_coordinate_system).

## To-Do

- Further improve fragment shader
- Investigate more performance improvements
- Better viewport calculation/zoom controls
- Test on a wider variety of 3D models

## References

- [tinyrenderer](https://github.com/ssloy/tinyrenderer)

## Some of My Other Terminal Projects

- [`tuibox`](https://github.com/Cubified/tuibox):  A single-header terminal UI (TUI) library in C.
- [`vt100utils`](https://github.com/Cubified/vt100utils):  A graphics escape sequence parser in C.
- [`bdfedit`](https://github.com/Cubified/bdfedit):  A bitmap font editor.
