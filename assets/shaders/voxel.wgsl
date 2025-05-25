struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) block_type: u32,
};

struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) block_type: u32,
}

struct FragmentOutput {
    @location(0) color: vec4<f32>,
}

const texture_step: f32 = 1./16.;

// we can import items from shader modules in the assets folder with a quoted path
const COLOR_MULTIPLIER: vec4<f32> = vec4<f32>(1.0, 1.0, 1.0, 0.5);

@group(2) @binding(0) var<uniform> material_color: vec4<f32>;
@group(2) @binding(1) var material_color_texture: texture_2d<f32>;
@group(2) @binding(2) var material_color_sampler: sampler;

const light: vec3<f32> = vec3(-0.57735027, 0.57735027, 0.57735027);

@fragment
fn fragment(
    in: VertexOutput,
) -> @location(0) vec4<f32> {
    let world_normal = normalize( cross( dpdy( in.world_position.xyz ), dpdx( in.world_position.xyz ) ) );

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
    var color = vec4(0.);
    let x = (in.block_type - 1) % 16;
    let y = (in.block_type - 1) / 16;
    var uvx = f32(x) / 16.;
    var uvy = (1.+f32(y)) / 16.;

    var axis: f32;
    if abs(world_normal.y) < 0.5 {
        axis = in.world_position.y % 1;
    } else {
        axis = in.world_position.z % 1;
    };
    if axis < 0 {
        axis += 1;
    }
    uvy -= (axis * texture_step);

    if abs(world_normal.x) < 0.5 {
        axis = in.world_position.x % 1;
    } else {
        axis = in.world_position.z % 1;
    };
    if axis < 0 {
        axis += 1;
    }
    uvx += axis * texture_step;

    if in.block_type == 4 && world_normal.y > 0.5 {
        uvx += 11. / 16.;
        uvy += 4. / 16.;
    }

    // if (world_normal.y) < 0.5 {
    //     var prog_y = in.world_position.y % 1;
    //     if prog_y < 0 {
    //         prog_y += 1.;
    //     }
    //     uvy -= (prog_y * texture_step);
    //     var axis: f32;
    //     if abs(world_normal.x) > 0.1 {
    //         axis = in.world_position.z % 1;
    //     } else {
    //         axis = in.world_position.x % 1;
    //     }
    //     if axis < 0 {
    //         axis += 1.;
    //     }
    //     uvx += axis * texture_step;
    //     color.x = uvx;
    //     color.y = uvy;
    // }

    // if in.uv.x != 0. {
    //     color.z = 1;
    // }


    // return color;

    return material_color * textureSample(material_color_texture, material_color_sampler, vec2(uvx, uvy)) * COLOR_MULTIPLIER * dp;
}

#import bevy_pbr::{
    in_bindings::in,
    mesh_functions,
    skinning,
    morph::morph,
    view_transformations::position_world_to_clip,
}

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.block_type = vertex.block_type;
    let in_world_from_local = mesh_functions::get_world_from_local(vertex.instance_index);

    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = in_world_from_local;


    /// set pos
    out.world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.world_position.x = round(out.world_position.x);
    out.world_position.y = round(out.world_position.y);
    out.world_position.z = round(out.world_position.z);
    out.position = position_world_to_clip(out.world_position.xyz);
    /// end set pos

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = mesh_functions::get_visibility_range_dither_level(
        vertex.instance_index, mesh_world_from_local[3]);
#endif

    return out;
}