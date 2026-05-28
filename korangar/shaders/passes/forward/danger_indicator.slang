#language slang 2026

import globals;
import forward;

struct DangerDecalTile {
    float4 upper_left;
    float4 upper_right;
    float4 lower_left;
    float4 lower_right;
};

struct DangerVertexOutput {
    float4 position : SV_Position;
    [[vk::location(0)]] var texture_coordinates: float2;
};

[[vk::binding(0, 0)]] var global_uniforms: ConstantBuffer<GlobalUniforms>;
[[vk::binding(1, 0)]] var nearest_sampler: SamplerState;
[[vk::binding(0, 2)]] var texture: Texture2D;
[[vk::binding(1, 2)]] var decal_tiles: StructuredBuffer<DangerDecalTile>;

[[shader("vertex")]]
func vs_main(uint vertex_index : SV_VulkanVertexID) -> DangerVertexOutput {
    // We draw 6 * tile_count vertices in a single instance to avoid needing
    // the DrawParameters capability (SV_InstanceID). Derive the tile and the
    // local vertex from the flat index.
    let tile_index = vertex_index / 6;
    let local_vertex = vertex_index % 6;
    let tile = decal_tiles[tile_index];
    let world_position = position_data(tile, local_vertex);

    var output: DangerVertexOutput;
    output.position = mul(global_uniforms.view_projection, world_position);
    output.texture_coordinates = uv_data(local_vertex);
    return output;
}

[[shader("pixel")]]
func fs_main(input: DangerVertexOutput) -> float4 {
    let diffuse_color = texture.Sample(nearest_sampler, input.texture_coordinates);
    let result = diffuse_color * global_uniforms.danger_color;
    if (result.a < 0.01) {
        discard;
    }
    return result;
}

[ForceInline]
func position_data(tile: DangerDecalTile, vertex_index: uint) -> float4 {
    switch (vertex_index) {
        case 0:
            return tile.upper_left;
        case 1:
            return tile.upper_right;
        case 2:
            return tile.lower_left;
        case 3:
            return tile.lower_left;
        case 4:
            return tile.upper_right;
        default:
            return tile.lower_right;
    }
}

[ForceInline]
func uv_data(vertex_index: uint) -> float2 {
    let index = 1 << vertex_index;
    let x = float((index & 0x2C) != 0);
    let y = float((index & 0x32) != 0);
    return float2(x, y);
}
