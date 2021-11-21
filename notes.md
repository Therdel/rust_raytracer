
# ToDo

- [x] port initial exercise1 project structure
    - [x] copy LICENSE
    - [x] learn to create (multi-level) Rust modules to translate project structure
    - [x] third party libraries
        - [x] **glm**: glm = "0.2.3"
        - [x] **stb**: stb = "0.3.2"
        - [x] **catch2**: Won't do, testing is integrated into rust
    - [x] test code
        - [x] structure
        - [x] test logic
- [x] port intersection tests
    - [x] sphere
    - [x] plane
    - [x] triangle
    - [x] unify surface acne compensation
- [x] scene config
    - [x] hardcoded
    - [x] parse from string
    - [x] parse from file
    - [x] primitives
        - [x] sphere
        - [x] plane
        - [x] triangle

- [x] gouraud shading
- [x] use typescript


- [ ] solve fetching files from WASM
  - using rust callback?
  - using js api in rust?
  - what about async?
## Web state machines
1. View holds State-Object, offers Interface, delegates to object
2. State-Objects hold View
3. Split into
   - View { canvas, draw buffer to canvas, animation frames, labels }
   - Controller { button, select, touch event
### SM Main
1. **Init**
   - Init controls
      - _Main deactivates canvas listener & scene selector_
   - Init workers
     - Workers send init
     - Main waits for all inits
   - _Main sends Event: SceneDefault (scene URL|JSON, resolution)_
     - Workers parse Scene and generate BVH
2. **Render-loop** (Single-pass)
   1. **Await worker paints**
      - Workers paint and response with buffers
      - Main paints buffers into canvas
      - Main awaits all responses
      - Main requires animation frame
   2. **Await control input**
      - _Main re-activates canvas listener & scene selector_
      - _Main sends Event Resize(_resolution_)_
         - Main resizes worker-buffers
         - Workers redo Raytracer world-to-screen
      - _Main sends Event Turn(_touch|mouse drag start/end canvas coords_)_
         - Workers calculate + perform camera rotation
         - Workers re-generate Raytracer
      - _Main sends Event: SceneSelect (scene URL|JSON, resolution)_
         - Workers parse Scene and generate BVH
      - _Main deactivates canvas listener & scene selector_
      - Send control event to worker

### Live render
The image could be rendered in multiple resolutions from coarse to fine,
enabling a more responsive experience
- [ ] Adam7 Interlacing
- [ ] ? Quad-Tree based LOD-ish
- [ ] Cancellation during rendering of intermediates - even more responsiveness
    
### Open questions
- [x] When to block / discard events?
  - Like disabling the render button during rendering, drag or resize events
  - Discard events between Main Event send and receiving all responses 
- [ ] Pre-generate deserialized BVHs
- [ ] Main could pre-fetch all available models
- [ ] BVH lib could be stripped out