struct Vertex {
    @builtin(instance_index) instance_index: u32,
#ifdef VERTEX_POSITIONS
    @location(0) position: vec3<f32>,
#endif
#ifdef VERTEX_NORMALS
    @location(1) normal: vec3<f32>,
#endif
#ifdef VERTEX_UVS_A
    @location(2) uv: vec2<f32>,
#endif
#ifdef VERTEX_UVS_B
    @location(3) uv_b: vec2<f32>,
#endif
#ifdef VERTEX_TANGENTS
    @location(4) tangent: vec4<f32>,
#endif
#ifdef VERTEX_COLORS
    @location(5) color: vec4<f32>,
#endif
#ifdef SKINNED
    @location(6) joint_indices: vec4<u32>,
    @location(7) joint_weights: vec4<f32>,
#endif
#ifdef MORPH_TARGETS
    @builtin(vertex_index) index: u32,
#endif
};

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    #ifdef VERTEX_UVS_A
        @location(2) uv: vec2<f32>,
    #endif
    #ifdef VERTEX_UVS_B
        @location(3) uv_b: vec2<f32>,
    #endif
    #ifdef VERTEX_TANGENTS
        @location(4) world_tangent: vec4<f32>,
    #endif
    #ifdef VERTEX_COLORS
        @location(5) color: vec4<f32>,
    #endif
    #ifdef VERTEX_OUTPUT_INSTANCE_INDEX
        @location(6) @interpolate(flat) instance_index: u32,
    #endif
    #ifdef VISIBILITY_RANGE_DITHER
        @location(7) @interpolate(flat) visibility_range_dither: i32,
    #endif
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

// we can import items from shader modules in the assets folder with a quoted path
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;

const light: vec3<f32> = vec3(-0.57735027, 0.57735027, 0.57735027);

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    let world_normal = normalize( cross( dpdy( mesh.world_position.xyz ), dpdx( mesh.world_position.xyz ) ) );

    // let dp = max(0., dot(world_normal, light));
    var dp = 0.7;

    if world_normal.y > 0.2 {
        dp = 1.;
    } else if world_normal.y < -0.2 {
        dp = 0.5;
    } else {
        if world_normal.x > 0.2 {
            dp += 0.05;   
        } else if world_normal.x < -0.2 {
            dp -= 0.1;
        }
        if world_normal.z > 0.2 {
            dp += 0.1;
        } else if world_normal.z < -0.2 {
            dp -= 0.05;
        }
    }

    return material_color * textureSample(material_color_texture, material_color_sampler, mesh.uv) * COLOR_MULTIPLIER * dp;
}