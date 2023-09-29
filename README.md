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

## To-Do

- Improve fragment shader (WIP code is commented out)

## References

- [tinyrenderer](https://github.com/ssloy/tinyrenderer)
