pub const TILE_VERTICES: &'static [u8] = include_bytes!("../meshes/vertex_data");
pub const TILE_INDICES: &'static [u8] = include_bytes!("../meshes/index_data");

pub fn get_map_data(id: u32) -> (&'static [u8], &'static [u8]) {
    (TILE_VERTICES, TILE_INDICES)
}
