@group(0) @binding(0)
var<storage, read_write> canvas: array<u32>;

@group(0) @binding(1)
var<uniform> screen_dimensions: vec2u;

@group(0) @binding(2)
var<uniform> screen_to_world: mat4x4f;

fn make_color(red: u32, green: u32, blue: u32, alpha: u32) -> u32 {
    let color: u32 = (alpha << 24) + (blue << 16) + (green << 8) + red;
    return color;
}

struct Ray {
    origin: vec3f,
    direction: vec3f
}

fn generate_primary_ray(screen_coordinate: vec2f, screen_to_world: mat4x4f) -> Ray {
    let p_screen = vec4f(screen_coordinate.x, screen_coordinate.y, 0, 1);
    // TODO: Document that NDC "looks" in *positive* z-axis. Document wrong viewing direction
    //       Has to do with how *WE* define the z-range.
    //       source: https://www.reddit.com/r/wgpu/comments/tilvas/comment/iyo1ml5
    // TODO: Document that this is *always* in camera view direction. (NDC)
    let p_screen_forward = p_screen + vec4f(0, 0, -1, 0);

    let p_world = screen_to_world * p_screen;
    let p_world_forward = screen_to_world * p_screen_forward;

    let p_world_inhomogeneous = (p_world / p_world.w).xyz;
    let p_world_forward_inhomogeneous = (p_world_forward / p_world_forward.w).xyz;

    let direction = p_world_forward_inhomogeneous - p_world_inhomogeneous;
    let direction_normalized = normalize(direction);

    return Ray(p_world_inhomogeneous, direction_normalized);
}

@compute
@workgroup_size(1)
fn main(@builtin(global_invocation_id) global_id: vec3u) {
    // second step: colored direction
    let max_y_index = screen_dimensions.y - 1;
    let y_inverted = max_y_index - global_id.y;
    let color_offset = y_inverted * screen_dimensions.x + global_id.x;

    let screen_coordinate = vec2f(global_id.xy);
    let ray = generate_primary_ray(screen_coordinate, screen_to_world);

    let dir_mapped_0_1 = (ray.direction+1)*0.5;
    let dir_color = vec3u(dir_mapped_0_1 * 255);
    let color = make_color(dir_color.x, dir_color.y, dir_color.z, 255);
    canvas[color_offset] = color;

    // final step: rendering
    // TODO
}
