# IVRIGST - Indirect Volume Rendering and Interactive General-purpose Shader Tool

<a href="https://github.com/stisol/rmedvis/actions/workflows/main.yml">
  <img src="https://github.com/stisol/rmedvis/actions/workflows/main.yml/badge.svg"/>
</a>

IVRIGST is a medical visualization tool for rendering blood vessel meshes for research purposes, written in Rust.

![Demonstration image](demo.png)

## Running the application

Place any models you wish to visualize in the `assets` directory in the standard `obj` format. Vertex colors are supported, but textures are not. When ready, start the application with:


```sh
cargo run --release
```

Shader files in the `shaders` directory are hot-reloaded and can be edited freely while the application is running.

Documentation for the application can be compiled using:

```sh
cargo doc --open --document-private-items
```

## License

This software is licensed under the License Zero Prosperity Public License 3.0.0. This essentially means you may use the software as you wish for non-commercial purposes.
