const NUMERIC_ERROR_COMPENSATION_OFFSET: f32 = 1e-4;

// const DEPTH_MAP_SCALE: f32 = 1.0 / 10.0;
// const DEPTH_MAP_SCALE: f32 = 1.0 * 200.0;
const DEPTH_MAP_SCALE: f32 = 0.85;

@group(0) @binding(0)
var<storage, read_write> canvas: array<u32>;

@group(0) @binding(1)
var<uniform> screen_dimensions: vec2u;

@group(0) @binding(2)
var<uniform> screen_to_world: mat4x4f;

@group(0) @binding(3)
var<storage, read> spheres: array<Sphere>;

/////////////////////

struct Sphere {
    center: vec3f,
    radius: f32,
    material: u32,
    _padding: array<u32, 3>,
}

fn sphere_normal(sphere: Sphere, surface_point: vec3f) -> vec3f {
    let surface_normal = surface_point - sphere.center;
    return normalize(surface_normal);
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
    color: ColorRgb
}

fn option_color_rgb_none() -> OptionColorRgb {
    return OptionColorRgb(false, vec3f(0.0));
}

fn option_color_rgb_add(lhs: OptionColorRgb, rhs: OptionColorRgb) -> OptionColorRgb {
    if lhs.is_some {
        if rhs.is_some {
            return OptionColorRgb(true, lhs.color + rhs.color);
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

fn map_direction_to_color_rgb(ray: Ray) -> ColorRgb {
    /// all components of the normalized vector mapped to [0, 2]
    let dir_mapped_0_2 = ray.direction + 1.0;
    /// all components of the vector mapped to [0, 1] - interpretable as RGB
    let dir_mapped_rgb = dir_mapped_0_2*0.5;
    return dir_mapped_rgb;
}

fn depth_map(ray: Ray) -> OptionColorRgb {
    let option_hitpoint = intersect_scene(ray);

    if (option_hitpoint.is_some) {
        // let brightness = u32(option_hitpoint.hitpoint.t * DEPTH_MAP_SCALE);
        // let color = color_rgb_quantize(brightness, brightness, brightness, 255);
        let brightness = option_hitpoint.hitpoint.t * DEPTH_MAP_SCALE;
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
    hitpoint: Hitpoint,
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

fn intersect_scene(ray: Ray) -> OptionHitpoint {
    var closest_hitpoint = option_hitpoint_none();

    let spheres_len = spheres_len();
    for (var i: u32 = 0u; i < spheres_len; i += 1u) {
        let sphere = spheres_get(i);

        let option_hitpoint = intersect_sphere(sphere, ray);
        if (option_hitpoint.is_some) {
            let hitpoint_is_closer = closest_hitpoint.is_some &&
                option_hitpoint.hitpoint.t < closest_hitpoint.hitpoint.t;
            let hitpoint_is_first = !closest_hitpoint.is_some;
            if (hitpoint_is_first || hitpoint_is_closer) {
                closest_hitpoint.is_some = true;
                closest_hitpoint.hitpoint = option_hitpoint.hitpoint;
            }
        }
    }

    return closest_hitpoint;
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

// TODO: Use canvas context for output https://gpuweb.github.io/gpuweb/explainer/#canvas-output
fn set_pixel(screen_coordinate: vec2u, color_rgb: ColorRgb) {
    let max_y_index = screen_dimensions.y - 1u;
    let y_inverted = max_y_index - screen_coordinate.y;
    let pixel_offset = y_inverted * screen_dimensions.x + screen_coordinate.x;

    let quantized_color = color_rgb_quantize(color_rgb, 255u);
    canvas[pixel_offset] = quantized_color;
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    let screen_coordinate = global_id.xy;

    // TODO: The y coord isn't inverted - that's why the y object's y coords are flipped
    let ray = generate_primary_ray(vec2f(screen_coordinate), screen_to_world);

    let option_color_rgb: OptionColorRgb = depth_map(ray);

    // TODO: Use canvas context for output https://gpuweb.github.io/gpuweb/explainer/#canvas-output
    if (option_color_rgb.is_some) {
        set_pixel(screen_coordinate, option_color_rgb.color);
    } else {
        let color_rgb = map_direction_to_color_rgb(ray);
        set_pixel(screen_coordinate, color_rgb);
    }
}
