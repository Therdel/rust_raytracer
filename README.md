# Whitted Raytracer in Rust
### Try it on [Github Pages](https://therdel.github.io/rust_raytracer)!
- translated previously built university project from C++ to Rust
- native app and webapp (WASM)


![](renders/reflective_bunnies.png)
![](renders/mirror-sphere_no-reflective-dimming.png)
![](renders/infinity_santa.png)

### TODO
[x] rename "exercise1" shit
[ ] web_app: put wasm-bindgen into build script
[ ] web_app: copy resources within build script - https://github.com/sotrh/learn-wgpu/blob/0.13/code/intermediate/tutorial12-camera/build.rs
[ ] Cheat obj --> loader
[ ] Load cheat objects from main?
[ ] fetch objs from workers
[ ] do cheat obj BVHs in seperate worker, async, while displaying cornell in parallel
    [ ] BVH serde to pass between workers
        --> bitcode crate: https://docs.rs/bitcode/latest/bitcode/
    [ ] pre-build BVHs, deploy in GH-pages
        --> bitcode crate: https://docs.rs/bitcode/latest/bitcode/

### TODO GPGPU
[x] read buffer into canvas
[x] Materials Arc --> Material IDs
[ ] Directly couple canvas, stop GPU<>CPU copying
[ ] Change canvas color Endianness("Canvas Format") based on device preference
    [source](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API#get_and_configure_the_canvas_context)
[ ] Experiment: Put into worker, measure time (hopefully fix issues with wasm-bindgen race)

### Learnt
- The WGSL storage read buffers that aren't referenced in the rest of the shader are optimized away
  to the point where there's an incompatibility between the bindgroup layouts.
- The "Canvas Format" (Color endianness) may not be RGBA but also GBRA:
  [source](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API#get_and_configure_the_canvas_context)