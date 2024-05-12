
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


- [ ] parallelize worker startup
    - problem: worker startup is serialized, as was scene fetching
- [ ] solve fetching files
    - disable SharedArrayBuffer & use OffScreenCanvases as replacement
    - if we can 
- [ ] generic actions batching
    - reason: A: sending messages for every action (e.g. turn_camera) kills responsiveness - e.g. calculating new frames at 60fps might not be possible. This increases Latency and indeterminism.
      state-of-the-art in turn_camera is keeping action messages until workers respond as ready, then transmit a single message that represents the change of all untransmitted actions (technically, we only send that message when the user triggers it, we don't queue).
      problem currently is, that UI actions (scene-select, resize, etc.) must be disabled during worker rendering
         --> that entails disabling-logic which is cumbersome
         --> and impossible-to-handle error scenarios, when messages couldn't be handled/sent (even though they could!)
      solution: generalize the turn_camera mechanism
          - UI state changes (canvas resize, turn_camera, scene change, mesh placement) are tracked until the next time workers are input-ready again. Events are merged, to lose unnecessary updates (2 turns merge to 1, 2 successive scene selects merge into the newest, resize merges into newest) - that way, responsiveness is ensured.
          - A: once all workers report ready, the merged message is sent
          - B: workers pull the current changeset on their own, saving one message
              [ ] how to ensure all workers see the same updates? Their state mustn't diverge
- [ ] remove the buffer-stitching on main as a potential perf improvement
    - state-of-the-art: workers render to SharedArrayBuffers in parallel, send a message to main once they're done, main stitches buffers together, line-by-line
    - idea: instead of one, use n stacked canvases belonging to a worker each. Workers receive an OffscreenCanvas (in major browsers since 2023-03) which they draw to in parallel. They can even request animation frames, to render continuously and *without message feedback to main*
        - flow control may still be implemented using feedback, this won't entail stitching on main, though.
   
  
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