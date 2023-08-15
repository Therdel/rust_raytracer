const NUMERIC_ERROR_COMPENSATION_OFFSET: f32 = 1e-4;

const DEPTH_MAP_SCALE: f32 = 0.85;

const MAX_RAY_RECURSION_DEPTH: u32 = 10u;
const REFLECTION_DIM_FACTOR: f32 = 0.8;

const MATERIAL_TYPE_PHONG: u32 = 0u;
const MATERIAL_TYPE_REFLECT_AND_PHONG: u32 = 1u;
const MATERIAL_TYPE_REFLECT_AND_REFRACT: u32 = 2u;

@group(0) @binding(0) var<storage, read_write> canvas: array<u32>;
@group(0) @binding(1) var<uniform> screen_dimensions: vec2u;
@group(0) @binding(2) var<uniform> screen_to_world: mat4x4f;

@group(0) @binding(3) var<storage, read> lights: array<Light>;
@group(0) @binding(4) var<storage, read> materials: array<Material>;

@group(0) @binding(5) var<storage, read> spheres: array<Sphere>;

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

struct Sphere {
    center: vec3f,
    radius: f32,
    material: u32,
    _padding: array<u32, 3>,
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
        let brightness = option_hitpoint.value.t * DEPTH_MAP_SCALE;
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
            //     // TODO: put background
            //     reflection_color = map_direction_to_color_rgb(reflected_ray);
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
        set_pixel(screen_coordinate, option_color_rgb.value);
    } else {
        let color_rgb = map_direction_to_color_rgb(ray);
        set_pixel(screen_coordinate, color_rgb);
    }

    let _ensure_bindings_exist = lights_len() + materials_len() + spheres_len();
}
