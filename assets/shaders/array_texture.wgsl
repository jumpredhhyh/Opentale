#import bevy_pbr::{
    forward_io::{Vertex, VertexOutput, FragmentOutput},
    mesh_view_bindings::view,
    pbr_fragment::pbr_input_from_standard_material,
    pbr_types::{STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT, PbrInput, pbr_input_new},
    pbr_functions::{alpha_discard, apply_pbr_lighting, main_pass_post_lighting_processing},
    mesh_functions::{get_world_from_local, mesh_position_local_to_world, mesh_tangent_local_to_world, get_visibility_range_dither_level, mesh_normal_local_to_world, get_tag},
    skinning::{skin_model, skin_normals},
    view_transformations::position_world_to_clip,
    pbr_bindings,
}
#import bevy_core_pipeline::tonemapping::tone_mapping

@group(2) @binding(100) var my_array_texture: texture_2d_array<f32>;
@group(2) @binding(101) var my_array_texture_sampler: sampler;

struct VertexCustom {
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
    @location(8) texture_id: u32
};

struct CustomVertexOutput {
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
    @location(8) texture_index: u32,
}

@vertex
fn vertex(vertex_custom: VertexCustom) -> CustomVertexOutput {
    var vertex_no_morph: Vertex;
    vertex_no_morph.instance_index = vertex_custom.instance_index; 
#ifdef VERTEX_POSITIONS
    vertex_no_morph.position = vertex_custom.position; 
#endif
#ifdef VERTEX_NORMALS
    vertex_no_morph.normal = vertex_custom.normal; 
#endif
#ifdef VERTEX_UVS_A
    vertex_no_morph.uv = vertex_custom.uv; 
#endif
#ifdef VERTEX_UVS_B
    vertex_no_morph.uv_b = vertex_custom.uv_b; 
#endif
#ifdef VERTEX_TANGENTS
    vertex_no_morph.tangent = vertex_custom.tangent; 
#endif
#ifdef VERTEX_COLORS
    vertex_no_morph.color = vertex_custom.color; 
#endif
#ifdef SKINNED
    vertex_no_morph.joint_indices = vertex_custom.joint_indices; 
    vertex_no_morph.joint_weights = vertex_custom.joint_weights; 
#endif
#ifdef MORPH_TARGETS
    vertex_no_morph.index = vertex_custom.index; 
#endif

    var out: VertexOutput;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

    let mesh_world_from_local = get_world_from_local(vertex_no_morph.instance_index);

#ifdef SKINNED
    var world_from_local = skin_model(
        vertex.joint_indices,
        vertex.joint_weights,
        vertex_no_morph.instance_index
    );
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var world_from_local = mesh_world_from_local;
#endif

#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skin_normals(world_from_local, vertex.normal);
#else
    out.world_normal = mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_position_local_to_world(world_from_local, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS_A
    out.uv = vertex.uv;
#endif
#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_tangent_local_to_world(
        world_from_local,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    out.instance_index = vertex_no_morph.instance_index;
#endif

#ifdef VISIBILITY_RANGE_DITHER
    out.visibility_range_dither = get_visibility_range_dither_level(
        vertex_no_morph.instance_index, mesh_world_from_local[3]);
#endif

    var custom_out: CustomVertexOutput;
    custom_out.position = out.position;
    custom_out.world_position = out.world_position;
    custom_out.world_normal = out.world_normal;
#ifdef VERTEX_UVS_A
    custom_out.uv = out.uv;
#endif
#ifdef VERTEX_UVS_B
    custom_out.uv_b = out.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    custom_out.world_tangent = out.world_tangent;
#endif
#ifdef VERTEX_COLORS
    custom_out.color = out.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    custom_out.instance_index = out.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    custom_out.visibility_range_dither = out.visibility_range_dither;
#endif

    custom_out.texture_index = vertex_custom.texture_id;

    return custom_out;
}

@fragment
fn fragment(
    in_custom: CustomVertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    var in: VertexOutput;
    in.position = in_custom.position;
    in.world_position = in_custom.world_position;
    in.world_normal = in_custom.world_normal;
#ifdef VERTEX_UVS_A
    in.uv = in_custom.uv;
#endif
#ifdef VERTEX_UVS_B
    in.uv_b = in_custom.uv_b;
#endif
#ifdef VERTEX_TANGENTS
    in.world_tangent = in_custom.world_tangent;
#endif
#ifdef VERTEX_COLORS
    in.color = in_custom.color;
#endif
#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    in.instance_index = in_custom.instance_index;
#endif
#ifdef VISIBILITY_RANGE_DITHER
    in.visibility_range_dither = in_custom.visibility_range_dither;
#endif

    var texture_index = in_custom.texture_index;

    /// generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    // pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    pbr_input.material.base_color = textureSample(my_array_texture, my_array_texture_sampler, in.uv, texture_index);
#ifdef VERTEX_COLORS
    pbr_input.material.base_color = pbr_input.material.base_color * in.color;
#endif


    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    return out;
}

#ifdef MORPH_TARGETS
fn morph_vertex(vertex_in: Vertex) -> Vertex {
    var vertex = vertex_in;
    let first_vertex = mesh[vertex.instance_index].first_vertex_index;
    let vertex_index = vertex.index - first_vertex;

    let weight_count = bevy_pbr::morph::layer_count();
    for (var i: u32 = 0u; i < weight_count; i ++) {
        let weight = bevy_pbr::morph::weight_at(i);
        if weight == 0.0 {
            continue;
        }
        vertex.position += weight * morph(vertex_index, bevy_pbr::morph::position_offset, i);
#ifdef VERTEX_NORMALS
        vertex.normal += weight * morph(vertex_index, bevy_pbr::morph::normal_offset, i);
#endif
#ifdef VERTEX_TANGENTS
        vertex.tangent += vec4(weight * morph(vertex_index, bevy_pbr::morph::tangent_offset, i), 0.0);
#endif
    }
    return vertex;
}
#endif
