const NUMERIC_ERROR_COMPENSATION_OFFSET: f32 = 1e-4;

const DEPTH_MAP_EXP_BASE: f32 = 2.0f;
const DEPTH_MAP_BRIGHTNESS_SCALE: f32 = 1.5f;

const MAX_RAY_RECURSION_DEPTH: u32 = 10u;
const REFLECTION_DIM_FACTOR: f32 = 0.8;

const MATERIAL_TYPE_PHONG: u32 = 0u;
const MATERIAL_TYPE_REFLECT_AND_PHONG: u32 = 1u;
const MATERIAL_TYPE_REFLECT_AND_REFRACT: u32 = 2u;

@group(0) @binding(0) var<storage, read_write> canvas: array<u32>;
@group(0) @binding(1) var<uniform> camera: Camera;
@group(0) @binding(2) var<uniform> background: Background;

@group(0) @binding(3) var<storage, read> lights: array<Light>;
@group(0) @binding(4) var<storage, read> materials: array<Material>;

/// Fixed-size array to reduce amount of needed storage buffers
@group(0) @binding(5) var<uniform> planes_and_triangles: PlanesAndTriangles;
@group(0) @binding(6) var<storage, read> spheres: array<Sphere>;
@group(0) @binding(7) var<storage, read> mesh_triangles: array<Triangle>;
@group(0) @binding(8) var<storage, read> mesh_bvh_nodes: array<BVHNode>;
@group(0) @binding(9) var<storage, read> meshes: array<Mesh>;
@group(0) @binding(10) var<storage, read> mesh_instances: array<MeshInstance>;

/////////////////////

struct Camera {
    screen_to_world: mat4x4f,
    screen_dimensions: vec2u,
}

struct Background {
    solid_color: vec3f,
    background_type: u32,
}

/////////////////////

struct Light {
    position: vec4f,
    color: LightColor,
}

struct LightColor {
    ambient: vec3f,
    diffuse: vec3f,
    specular: vec3f,
}

/// TODO: Use pointers
fn lights_get(index: u32) -> Light {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return lights[index + 1u];
}

fn lights_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&lights) - 1u;
}

/////////////////////

struct Material {
    emissive: vec3f,
    ambient: vec3f,
    diffuse: vec3f,
    specular: vec3f,
    shininess: f32,

    material_type: u32,
    /// only set on material_type == ReflectAndRefract
    index_inner: f32,
    /// only set on material_type == ReflectAndRefract
    index_outer: f32,
    _padding: u32
}

/// TODO: Use pointers
fn materials_get(index: u32) -> Material {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return materials[index + 1u];
}

fn materials_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&materials) - 1u;
}

/////////////////////

struct PlanesAndTriangles {
    planes: array<Plane, 64>,
    triangles: array<Triangle, 64>,
    planes_len: u32,
    triangles_len: u32,
    padding0: u32,
    padding1: u32,
}

struct Plane {
    normal: vec3f,
    distance: f32,
    material: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

struct Triangle {
    vertices: array<vec3f, 3>,
    normals: array<vec3f, 3>,
    normal: vec3f,

    material: u32,
}

/// TODO: Use pointers
fn planes_get(index: u32) -> Plane {
    return planes_and_triangles.planes[index];
}

fn planes_len() -> u32 {
    return planes_and_triangles.planes_len;
}

/// TODO: Use pointers
fn triangles_get(index: u32) -> Triangle {
    return planes_and_triangles.triangles[index];
}

fn triangles_len() -> u32 {
    return planes_and_triangles.triangles_len;
}

/////////////////////

struct Sphere {
    center: vec3f,
    radius: f32,
    material: u32,
    _padding0: u32,
    _padding1: u32,
    _padding2: u32,
}

/// TODO: Use pointers
fn spheres_get(index: u32) -> Sphere {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return spheres[index + 1u];
}

fn spheres_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&spheres) - 1u;
}

fn sphere_normal(sphere: Sphere, surface_point: vec3f) -> vec3f {
    let surface_normal = surface_point - sphere.center;
    return normalize(surface_normal);
}

/////////////////////
/// TODO: Use pointers
fn mesh_triangles_get(index: u32) -> Triangle {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return mesh_triangles[index + 1u];
}

fn mesh_triangles_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&mesh_triangles) - 1u;
}

/////////////////////

struct BVHNode {
    aabb_min: vec3f,
    aabb_max: vec3f,
    is_leaf: u32,
    child_left_index: u32,
    child_right_index: u32,
    triangle_indices: array<u32, 5>,
    triangle_indices_len: u32,
}

/// TODO: Use pointers
fn mesh_bvh_nodes_get(index: u32) -> BVHNode {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return mesh_bvh_nodes[index + 1u];
}

fn mesh_bvh_nodes_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&mesh_bvh_nodes) - 1u;
}

/////////////////////

struct Mesh {
    triangle_indices_start: u32,
    triangle_indices_end: u32,
    bvh_node_indices_start: u32,
    bvh_node_indices_end: u32,
    bvh_max_depth: u32
}

/// TODO: Use pointers
fn meshes_get(index: u32) -> Mesh {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return meshes[index + 1u];
}

fn meshes_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&meshes) - 1u;
}

/////////////////////

struct MeshInstance {
    rotation_scale: mat4x4f,
    rotation_scale_inverse: mat4x4f,
    model: mat4x4f,
    model_inverse: mat4x4f,
    mesh_index: u32,
    material_override: u32,
    material_override_is_some: u32,
    _padding: u32,
}

/// TODO: Use pointers
fn mesh_instances_get(index: u32) -> MeshInstance {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed
    return mesh_instances[index + 1u];
}

fn mesh_instances_len() -> u32 {
    // the first element is always a dummy, as empty 
    // runtime-sized arrays aren't allowed.
    // so the actual length is 1 lower
    return arrayLength(&mesh_instances) - 1u;
}

/////////////////////

/////////////////////

/// RGB f32 color with range [0.0, 1.0]
alias ColorRgb = vec3f;

/// RGBA u8 color with range [0, 255]
alias ColorRgbaU8 = u32;

fn color_rgba_u8_new(red: u32, green: u32, blue: u32, alpha: u32) -> ColorRgbaU8 {
    // TODO: This layout may differ between devices
    // The "Canvas Format" (Color endianness) may not be RGBA but also GBRA:
    // [source](https://developer.mozilla.org/en-US/docs/Web/API/WebGPU_API#get_and_configure_the_canvas_context)
    let color: u32 = (alpha << 24u) + (blue << 16u) + (green << 8u) + red;
    return color;
}

fn color_rgb_quantize(color: ColorRgb, alpha: u32) -> ColorRgbaU8 {
    let clamped_color = clamp(color, vec3f(0.0), vec3f(1.0));
    let quantized_color = vec3u(clamped_color * 255.0);

    return color_rgba_u8_new(quantized_color.x, quantized_color.y, quantized_color.z, alpha);
}

struct OptionColorRgb {
    is_some: bool,
    value: ColorRgb
}

fn option_color_rgb_none() -> OptionColorRgb {
    return OptionColorRgb(false, vec3f(0.0));
}

fn option_color_rgb_add(lhs: OptionColorRgb, rhs: OptionColorRgb) -> OptionColorRgb {
    if lhs.is_some {
        if rhs.is_some {
            return OptionColorRgb(true, lhs.value + rhs.value);
        } else {
            return lhs;
        }
    } else {
        return rhs;
    }
}

/////////////////////

struct Ray {
    origin: vec3f,
    direction: vec3f
}

fn generate_primary_ray(screen_coordinate: vec2f, screen_to_world: mat4x4f) -> Ray {
    let p_screen = vec4f(screen_coordinate.x, screen_coordinate.y, 0.0, 1.0);
    // TODO: Document that NDC "looks" in *positive* z-axis. Document wrong viewing direction
    //       Has to do with how *WE* define the z-range.
    //       source: https://www.reddit.com/r/wgpu/comments/tilvas/comment/iyo1ml5
    // TODO: Document that this is *always* in camera view direction. (NDC)
    let p_screen_forward = p_screen + vec4f(0.0, 0.0, -1.0, 0.0);

    let p_world = screen_to_world * p_screen;
    let p_world_forward = screen_to_world * p_screen_forward;

    let p_world_inhomogeneous = (p_world / p_world.w).xyz;
    let p_world_forward_inhomogeneous = (p_world_forward / p_world_forward.w).xyz;

    let direction = p_world_forward_inhomogeneous - p_world_inhomogeneous;
    let direction_normalized = normalize(direction);

    return Ray(p_world_inhomogeneous, direction_normalized);
}

fn trace_background(ray: Ray) -> ColorRgb {
    if (background.background_type == 0) {
        // Background::SolidColor
        return background.solid_color;
    } else {
        // Background::ColoredDirection
        /// all components of the normalized vector mapped to [0, 2]
        let dir_mapped_0_2 = ray.direction + 1.0;
        /// all components of the vector mapped to [0, 1] - interpretable as RGB
        let dir_mapped_rgb = dir_mapped_0_2*0.5;
        return dir_mapped_rgb;
    }
}

fn depth_map(ray: Ray) -> OptionColorRgb {
    let option_hitpoint = intersect_scene(ray);

    if (option_hitpoint.is_some) {
        let brightness = pow(DEPTH_MAP_EXP_BASE, -option_hitpoint.value.t) * DEPTH_MAP_BRIGHTNESS_SCALE;
        let color = ColorRgb(brightness);
        return OptionColorRgb(true, color);
    } else {
        return option_color_rgb_none();
    }
}

struct Hitpoint {
    t: f32,
    position: vec3f,
    hit_normal: vec3f,
    position_for_refraction: vec3f,
    on_frontside: bool,

    material: u32,
}

struct OptionHitpoint {
    is_some: bool,
    value: Hitpoint,
}

fn option_hitpoint_none() -> OptionHitpoint {
    let hitpoint_none = Hitpoint(0.0,
                                 vec3f(0.0, 0.0, 0.0),
                                 vec3f(0.0, 0.0, 0.0),
                                 vec3f(0.0, 0.0, 0.0),
                                 false,
                                 0u);

    return OptionHitpoint(false, hitpoint_none);
}

struct OptionF32 {
    is_some: bool,
    value: f32,
}

fn utils_take_hitpoint_if_closer(closest_hitpoint: ptr<function, OptionHitpoint>,
                                 hitpoint:         ptr<function, OptionHitpoint>)  {
    if (*hitpoint).is_some {
        if (*closest_hitpoint).is_some {
            if (*hitpoint).value.t < (*closest_hitpoint).value.t {
                *closest_hitpoint = *hitpoint;
            }
        } else {
                *closest_hitpoint = *hitpoint;
        }
    }
}

fn intersect_scene(ray: Ray) -> OptionHitpoint {
    var closest_hitpoint = option_hitpoint_none();

    // TODO: Evaluating the length in the condition has huge (100's of ms) performance implications
    let planes_len = planes_len();
    for (var i: u32 = 0u; i < planes_len; i += 1u) {
        let plane = planes_get(i);

        var option_hitpoint: OptionHitpoint = intersect_plane(plane, ray);
        utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
    }

    let spheres_len = spheres_len();
    for (var i: u32 = 0u; i < spheres_len; i += 1u) {
        let sphere = spheres_get(i);

        var option_hitpoint: OptionHitpoint = intersect_sphere(sphere, ray);
        utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
    }

    // TODO: Evaluating the length in the condition has huge (100's of ms) performance implications
    let triangles_len = triangles_len();
    for (var i: u32 = 0u; i < triangles_len; i += 1u) {
        let triangle = triangles_get(i);

        var option_hitpoint = intersect_triangle(triangle, ray);
        utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
    }

    let mesh_instances_len = mesh_instances_len();
    for (var i: u32 = 0u; i < mesh_instances_len; i += 1u) {
        let mesh_instance = mesh_instances_get(i);

        var option_hitpoint = intersect_mesh_instance(mesh_instance, ray);
        utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
    }

    return closest_hitpoint;
}

fn intersect_plane(plane: Plane, ray: Ray) -> OptionHitpoint {
    var result = option_hitpoint_none();

    let n_dot_rdir = dot(plane.normal, ray.direction);
    let parallel = n_dot_rdir == 0.0;
    if !parallel {
        // t = d - N * rOrg
        //     ------------
        //       N * rDir
        let t = (plane.distance - dot(plane.normal, ray.origin))
            / n_dot_rdir;

        let does_intersect_in_ray_direction = t >= 0.0;
        if does_intersect_in_ray_direction {
            let hit_position = utils_ray_equation(ray, t);
            let hitpoint = create_hitpoint(t, hit_position, ray, plane.normal, plane.normal, plane.material);

            result = OptionHitpoint(true, hitpoint);
        }
    }
    return result;
}

fn intersect_sphere(sphere: Sphere, ray: Ray) -> OptionHitpoint {
    var result = option_hitpoint_none();

    // m = rOrg - C
    let m = ray.origin - sphere.center;
    // a = rDir * rDir
    let a = dot(ray.direction, ray.direction);
    // b = 2(m * rDir)
    let b = 2.0 * dot(m, ray.direction);
    // c = (m * m) - r²
    let c = dot(m, m) - sphere.radius*sphere.radius;

    // 4 rDir² (r² - (m - (m * rDir^ ) * rDir^ )² )
    // where rDir^ means normalized
    //
    // 4 dot(rDir, rDir)
    // * (pow(r, 2) - dot(m - dot(m, rDir^) * rDir^,
    //                    m - dot(m, rDir^) * rDir^)
    //   )
    let r_dir_norm = normalize(ray.direction);
    let discriminant =
        4.0 * dot(ray.direction, ray.direction)
        * (sphere.radius*sphere.radius
        - dot((m - r_dir_norm * dot(m, r_dir_norm)),
              (m - r_dir_norm * dot(m, r_dir_norm)))
    );

    var option_t = OptionF32(false, 0.0);
    if (discriminant == 0.0) {
        option_t = OptionF32(true, (-0.5 * b ) / a);
    } else if (discriminant > 0.0) {
        // calculate intersections
        // t0 = q / a
        // t1 = c / q
        //
        // where q = -0.5(b - sqrt(discriminant)  for b < 0
        //           -0.5(b + sqrt(discriminant)  otherwise
        var q = 0.0;
        if (b < 0.0) {
            q = -0.5 * (b - sqrt(discriminant));
        } else {
            q = -0.5 * (b + sqrt(discriminant));
        };
        let t0 = q / a;
        let t1 = c / q;


        if (t0 < 0.0 && t1 >= 0.0) {
            // first intersection behind ray origin, second valid
            option_t = OptionF32(true, t1);
        } else if (t1 < 0.0 && t0 >= 0.0) {
            // second intersection behind ray origin, first falid
            option_t = OptionF32(true, t0);
        } else {
            // either both behind ray origin (invalid) or both valid
            // take the first intersection in ray direction
            option_t = OptionF32(true, min(t0, t1));
        }
    };

    if option_t.is_some {
        let t = option_t.value;
        let does_intersect_in_ray_direction = t >= 0.0;
        if does_intersect_in_ray_direction {
            let hit_position = utils_ray_equation(ray, t);
            let normal = sphere_normal(sphere, hit_position);
            let hitpoint = create_hitpoint(t, hit_position, ray, normal, normal, sphere.material);

            result = OptionHitpoint(true, hitpoint);
        }
    }
    return result;
}

fn intersect_triangle(triangle: Triangle, ray: Ray) -> OptionHitpoint {        
        let e1 = triangle.vertices[1] - triangle.vertices[0];
        let e2 = triangle.vertices[2] - triangle.vertices[0];
        let q = cross(ray.direction, e2);
        let a = dot(e1, q);

        const EPSILON: f32 = 1e-5;
        if a > -EPSILON && a < EPSILON { return option_hitpoint_none(); }

        let f = 1.0/a;
        let s = ray.origin - triangle.vertices[0];
        let u = f * dot(s, q);
        if u < 0.0 { return option_hitpoint_none(); }

        let r = cross(s, e1);
        let v = f * dot(ray.direction, r);
        if v < 0.0 || u + v > 1.0 { return option_hitpoint_none(); }

        let t = f * dot(e2, r);
        if t < 0.0 { return option_hitpoint_none(); }

        let w = 1.0 - u - v;
        let hit_position = utils_ray_equation(ray, t);
        let hit_normal_gouraud = w * triangle.normals[0] + u * triangle.normals[1] + v * triangle.normals[2];
        let hit_normal_gouraud_normalized = normalize(hit_normal_gouraud);
        let hitpoint = create_hitpoint(t, hit_position, ray, triangle.normal, hit_normal_gouraud_normalized, triangle.material);

        return OptionHitpoint(true, hitpoint);
}

// TODO: optimize branching at the end
fn intersect_aabb(aabb_min: vec3f, aabb_max: vec3f, ray: Ray) -> bool {
    let dirfrac = vec3f(
        // r.dir is unit direction vector of ray
        1.0 / ray.direction.x,
        1.0 / ray.direction.y,
        1.0 / ray.direction.z,
    );
    // lb is the corner of AABB with minimal coordinates - left bottom, rt is maximal corner
    // r.org is origin of ray
    let lb = aabb_min;
    let rt = aabb_max;
    let t1 = (lb.x - ray.origin.x)*dirfrac.x;
    let t2 = (rt.x - ray.origin.x)*dirfrac.x;
    let t3 = (lb.y - ray.origin.y)*dirfrac.y;
    let t4 = (rt.y - ray.origin.y)*dirfrac.y;
    let t5 = (lb.z - ray.origin.z)*dirfrac.z;
    let t6 = (rt.z - ray.origin.z)*dirfrac.z;

    let tmin = max(max(min(t1, t2), min(t3, t4)), min(t5, t6));
    let tmax = min(min(max(t1, t2), max(t3, t4)), max(t5, t6));

    // if tmax < 0, ray (line) is intersecting AABB, but the whole AABB is behind us
    var _t = f32();
    if tmax < 0.0 {
        _t = tmax;
        return false;
    }

    // if tmin > tmax, ray doesn't intersect AABB
    if tmin > tmax {
        _t = tmax;
        return false;
    }

    _t = tmin;
    return true;
}

const STACK_LEN: u32 = 32;
struct BvhNodeStack {
    stack: array<u32, STACK_LEN>,
    stack_len: u32,
}

fn bvh_node_stack_new() -> BvhNodeStack {
    return BvhNodeStack(array<u32, STACK_LEN>(), 0);
}

fn bvh_node_stack_push(stack: ptr<function, BvhNodeStack>, node_index: u32) {
    let next_index = (*stack).stack_len;
    (*stack).stack[next_index] = node_index;
    (*stack).stack_len += 1u;
}

fn bvh_node_stack_pop(stack: ptr<function, BvhNodeStack>) -> u32 {
    let current_index = (*stack).stack_len - 1u;
    (*stack).stack_len -= 1u;
    return (*stack).stack[current_index];
}

fn intersect_bvh(bvh_node_indices_start: u32, bvh_node_indices_end: u32, ray: Ray) -> OptionHitpoint {
    var closest_hitpoint = option_hitpoint_none();

    let is_empty = bvh_node_indices_start >= bvh_node_indices_end;
    if is_empty {
        return option_hitpoint_none();
    }

    let root_index = bvh_node_indices_start;
    var stack: BvhNodeStack = bvh_node_stack_new();
    bvh_node_stack_push(&stack, root_index);
    while (stack.stack_len > 0u) {
        let node_index = bvh_node_stack_pop(&stack);
        let node = mesh_bvh_nodes_get(node_index);

        let did_hit_aabb = intersect_aabb(node.aabb_min, node.aabb_max, ray);
        // visualizes root AABB 👀
        // return debug_bool(did_hit_aabb);
        if (!did_hit_aabb) {
            continue;
        }

        let is_node = node.is_leaf == 0u;
        if (is_node) {
            bvh_node_stack_push(&stack, node.child_left_index);
            bvh_node_stack_push(&stack, node.child_right_index);
        } else {
            let triangle_indices_len = node.triangle_indices_len;
            for (var i = 0u; i < triangle_indices_len; i += 1u) {
                let triangle_index = node.triangle_indices[i];
                let triangle = mesh_triangles_get(triangle_index);
                
                var option_hitpoint: OptionHitpoint = intersect_triangle(triangle, ray);
                utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
            }
        }
    }

    return closest_hitpoint;
}

fn intersect_mesh(mesh: Mesh, ray: Ray) -> OptionHitpoint {
    const USE_BVH: bool = true;
    if (USE_BVH) {
        return intersect_bvh(mesh.bvh_node_indices_start, mesh.bvh_node_indices_end, ray);
    } else {
        let triangle_indices_len = mesh.triangle_indices_end - mesh.triangle_indices_start;
        if (triangle_indices_len > 0) {
            // return OptionHitpoint(Hiptoint(), true)
            var closest_hitpoint = option_hitpoint_none();
            for (var i: u32 = mesh.triangle_indices_start; i < mesh.triangle_indices_end; i += 1u) {
                let triangle = mesh_triangles_get(i);

                var option_hitpoint = intersect_triangle(triangle, ray);
                utils_take_hitpoint_if_closer(&closest_hitpoint, &option_hitpoint);
            }
            return closest_hitpoint;
        } else {
            return option_hitpoint_none();
        }
    }
}

fn transform_unhomogeneous(vector: vec3f, matrix: mat4x4f) -> vec3f {
    let homogeneous_transformed = matrix * vec4f(vector, 1.0);
    // no perspective divide needed as we're only using translate, scale & rotate
    return homogeneous_transformed.xyz; 
}

fn intersect_mesh_instance(instance: MeshInstance, ray: Ray) -> OptionHitpoint {
    let mesh = meshes_get(instance.mesh_index);

    if (DEPTH_MAP_LINEAR_FOR_DEBUGGING) {
        return intersect_mesh(mesh, ray);
    }

    // transform ray into model-local coordinate-system
    let transformed_origin = transform_unhomogeneous(ray.origin, instance.model_inverse);
    let transformed_direction = normalize(
        transform_unhomogeneous(ray.direction, instance.rotation_scale_inverse)
    );
    let transformed_ray = Ray(transformed_origin, transformed_direction);

    let option_hitpoint = intersect_mesh(mesh, transformed_ray);
    if (!option_hitpoint.is_some) {
        return option_hitpoint_none();
    }
    var hitpoint = option_hitpoint.value;

    // transform hitpoint back into world-local coordinate-system
    hitpoint.position = transform_unhomogeneous(hitpoint.position, instance.model);
    hitpoint.hit_normal = normalize(
        transform_unhomogeneous(hitpoint.hit_normal, instance.rotation_scale)
    );
    hitpoint.position_for_refraction = transform_unhomogeneous(hitpoint.position_for_refraction, instance.model);

    let t_in_world = distance(ray.origin, hitpoint.position);
    hitpoint.t = t_in_world;

    if (instance.material_override_is_some == 1u) {
        hitpoint.material = instance.material_override;
    }
    return OptionHitpoint(true, hitpoint);
}

fn create_hitpoint(t: f32, hit_position: vec3f, ray: Ray,
                   IN_surface_normal: vec3f, IN_hit_normal: vec3f,
                   material: u32) -> Hitpoint {
    let n_dot_rdir = dot(IN_surface_normal, ray.direction);
    let intersect_frontside = n_dot_rdir < 0.0;

    // invert normals when hitting the back or inside of the geometry
    var surface_normal = IN_surface_normal;
    var hit_normal = IN_hit_normal;
    if !intersect_frontside {
        surface_normal *= -1.0;
        hit_normal *= -1.0;
    }

    // compensate numeric error on intersection.
    // moves hitpoint along surface normal in direction of ray origin
    // this avoids cases where hitpoints numerically "sink through" the surface
    let offset = surface_normal * NUMERIC_ERROR_COMPENSATION_OFFSET;
    let hit_position_acne_compensated = hit_position + offset;

    // refractive ray begins on the other side of the geometry.
    // Preventing acne effects on this side, the acne compensation vector is
    // substracted from the hit position
    let hit_position_for_refraction = hit_position - offset;

    return Hitpoint(
        t,
        hit_position_acne_compensated,
        hit_normal,
        hit_position_for_refraction,
        intersect_frontside,
        material,
    );
}

fn utils_ray_equation(ray: Ray, t: f32) -> vec3f {
    return ray.origin + ray.direction * t;
}

fn raytrace(ray: Ray) -> OptionColorRgb {
    return raytrace_impl(ray, 0u);
}

fn raytrace_impl(ray: Ray, ray_recursion_depth: u32) -> OptionColorRgb {
    if ray_recursion_depth < MAX_RAY_RECURSION_DEPTH {
        let option_hitpoint = intersect_scene(ray);
        if option_hitpoint.is_some {
            return shade(ray, option_hitpoint.value, ray_recursion_depth);
        } else {
            return option_color_rgb_none();
        }
    } else {
        return option_color_rgb_none();
    }
}

// TODO: Move shades into functions
fn shade(ray: Ray, hitpoint: Hitpoint, ray_recursion_depth: u32) -> OptionColorRgb {
    let material = materials_get(hitpoint.material);
    var result = option_color_rgb_none();
    
    switch material.material_type {
        case MATERIAL_TYPE_PHONG: {
            var current_color: OptionColorRgb = option_color_rgb_none();

            let lights_len = lights_len();
            for (var i: u32 = 0u; i < lights_len; i += 1u) {
                let light = lights_get(i);

                let is_shadow = trace_shadow_ray(hitpoint.position, light);
                let radiance_color = radiance(ray, hitpoint, light, is_shadow);

                current_color = option_color_rgb_add(current_color, OptionColorRgb(true, radiance_color));
            }
            result = option_color_rgb_add(result, current_color);
        }
        case MATERIAL_TYPE_REFLECT_AND_PHONG: {
            result = OptionColorRgb(true, ColorRgb(0.0, 1.0, 0.0));

            // let option_reflection_color = raytrace_impl(reflected_ray, ray_recursion_depth + 1);
            // var reflection_color = vec3f();
            // if option_reflection_color.is_some {
            //     reflection_color = option_reflection_color.value;
            // } else{
            //     reflection_color = trace_background(reflected_ray);
            // }
            // result = option_color_rgb_add(result, OptionColorRgb(true, reflection_color * REFLECTION_DIM_FACTOR));
        }
        case MATERIAL_TYPE_REFLECT_AND_REFRACT: {
            result = OptionColorRgb(true, ColorRgb(0.0, 0.0, 1.0));
        }
        case default: {
            result = OptionColorRgb(true, ColorRgb(1.0, 0.0, 1.0));
        }
    }
    return result;
}

fn radiance(ray: Ray, hitpoint: Hitpoint, light: Light, is_shadow: bool) -> ColorRgb {
    let material = materials_get(hitpoint.material);
    let l = get_hitpoint_to_light_unit_vector(hitpoint, light);
    let n = hitpoint.hit_normal;
    let v = -ray.direction;
    let r = create_reflected_ray(l, n);

    let l_dot_n = max(dot(l, n), 0.0);
    let r_dot_v = max(dot(r, v), 0.0);

    let emissive = material.emissive;
    let ambient = light.color.ambient * material.ambient;
    var diffuse = ColorRgb(0);
    var specular = ColorRgb(0);
    if !is_shadow {
        diffuse = (light.color.diffuse * material.diffuse) * l_dot_n;
        specular = (light.color.specular * material.specular) * pow(r_dot_v, material.shininess);
    }

    return emissive + ambient + diffuse + specular;
}

fn trace_shadow_ray(world_pos: vec3f, light: Light) -> bool {
    let is_directional_light = light.position.w == 0;

    var direction_unnormalized = vec3f();
    if is_directional_light {
        direction_unnormalized = light.position.xyz;
    } else {
        let light_world_pos = (light.position / light.position.w).xyz;
        direction_unnormalized = light_world_pos - world_pos;
    }

    let direction = normalize(direction_unnormalized);

    let ray = Ray(world_pos, direction);

    var is_shadow = bool();
    let option_hitpoint = intersect_scene(ray);
    if option_hitpoint.is_some {
        let hitpoint = option_hitpoint.value;
        if is_directional_light {
            // any intersection puts shadow of infinitely distant (directional light)
            is_shadow = true;
        } else {
            let light_world_pos = (light.position / light.position.w).xyz;
            let distance_to_light = distance(ray.origin, light_world_pos);
            let ray_distance_travelled = hitpoint.t;

            is_shadow = ray_distance_travelled < distance_to_light;
        }
    } else {
        is_shadow = false;
    }

    return is_shadow;
}

fn create_reflected_ray(to_viewer: vec3f, normal: vec3f) -> vec3f {
    let V = to_viewer;
    let N = normal;
    return 2. * dot(N, V) * N - V;
}

fn get_hitpoint_to_light_unit_vector(hitpoint: Hitpoint, light: Light) -> vec3f {
    let is_directional_light = light.position.w == 0.0;

    var vector = vec3f(0);
    if is_directional_light {
        vector = light.position.xyz;
    } else {
        let light_world_pos = (light.position / light.position.w).xyz;
        vector = light_world_pos - hitpoint.position;
    }
    return normalize(vector);
}

// TODO: Use canvas context for output https://gpuweb.github.io/gpuweb/explainer/#canvas-output
fn set_pixel(screen_coordinate: vec2u, color_rgb: ColorRgb) {
    let max_y_index = camera.screen_dimensions.y - 1u;
    let y_inverted = max_y_index - screen_coordinate.y;
    let pixel_offset = y_inverted * camera.screen_dimensions.x + screen_coordinate.x;

    let quantized_color = color_rgb_quantize(color_rgb, 255u);
    canvas[pixel_offset] = quantized_color;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let screen_coordinate = global_id.xy;

    // TODO: The y coord isn't inverted - that's why the y object's y coords are flipped
    let ray = generate_primary_ray(vec2f(screen_coordinate), camera.screen_to_world);

    let option_color_rgb: OptionColorRgb = depth_map(ray);

    // TODO: Use canvas context for output https://gpuweb.github.io/gpuweb/explainer/#canvas-output
    if (option_color_rgb.is_some) {
        set_pixel(screen_coordinate, option_color_rgb.value);
    } else {
        let color_rgb = trace_background(ray);
        set_pixel(screen_coordinate, color_rgb);
    }

    let _ensure_bindings_exist = lights_len() + materials_len() + planes_len() +
                                 spheres_len() + triangles_len() + mesh_triangles_len() +
                                 mesh_bvh_nodes_len() + meshes_len() + mesh_instances_len();
}