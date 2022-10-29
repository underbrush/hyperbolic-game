pub mod permutation;
mod mapdata;

use nalgebra::{ Vector4, Matrix4 };

use self::permutation::*;

const SQ3: f32 = 1.73205080757;

pub struct Tile {
    code: GroupElt,
    vertices: &'static [u8],
    indices: &'static [u8],

    vbuf: Option<wgpu::Buffer>,
    ibuf: Option<wgpu::Buffer>,
    pos: Option<wgpu::Buffer>,
    bind_group: Option<wgpu::BindGroup>,
}

impl Tile {
    pub fn new(
        mut code: GroupElt,
    ) -> Tile {
        let (vertices, indices) = mapdata::get_map_data(code.get_id());
        code.make_repr();
        Self {
            code,
            vertices,
            indices,

            vbuf: None,
            ibuf: None,
            pos: None,
            bind_group: None,
        }
    }

    pub fn update_neighbors(
        &mut self,
        position: &[f32; 4],
        existing_codes: &[u32],
    ) -> (Vec<Tile>, Vec<u32>) {
        let pos = Vector4::from_column_slice(position);
        let mut corners = [
            Vector4::new(1.0, 1.0, 0.0, SQ3),
            Vector4::new(1.0, -1., 0.0, SQ3),
            Vector4::new(-1., -1., 0.0, SQ3),
            Vector4::new(-1., 1.0, 0.0, SQ3),
        ];
        let mut distances = [0.0; 4];

        let mut out = (Vec::<Tile>::new(), Vec::<u32>::new());

        for i in 0..4 {
            corners[i] = self.get_mat() * corners[i];
            distances[i] = (corners[i].w * pos.w
                - corners[i].xy().dot(&pos.xy())).acosh();
        }

        for i in 0..4 {
            if distances[(i) % 4] < distances[(i + 1) % 4] {
                let mut new_code = self.code.multiply(&TRANSLATION);

                if !existing_codes.contains(&new_code.get_id()) {
                    out.0.push(Self::new(new_code));
                    out.1.push(new_code.get_id());
                }
            }
            self.code.right_multiply_in_place(&ROTATION);
        }

        out
    }

    pub fn move_to(&mut self, code: GroupElt) -> Vec<GroupElt> {
        self.code = code;
        let mut out = Vec::<GroupElt>::new();

        for _ in 0..4 {
            let mut new_code = self.code.multiply(&TRANSLATION);
            new_code.make_repr();

            out.push(new_code);
            self.code.right_multiply_in_place(&ROTATION);
        }

        out
    }

    pub fn distance_from(&self, position: &[f32; 4],) -> f32 {
        let pos = Vector4::from_column_slice(position);
        let center = self.get_mat() * Vector4::new(0.0, 0.0, 0.0, 1.0);
        let distance = (center.w * pos.w
            - center.xy().dot(&pos.xy())).acosh();
        
        distance
    }

    pub fn get_vertices(&self) -> &[u8] { self.vertices }
    pub fn get_indices(&self) -> &[u8] { self.indices }
    pub fn get_mat(&self) -> Matrix4<f32>
        { self.code.get_matrix() }

    pub fn centered_code(&self) -> GroupElt
        { self.code.permute_only()}
    pub fn move_against(
        &self,
        pos: &[f32; 4],
        dir: &[f32; 3]
    ) -> ([f32; 4], [f32; 3]) {
        let pos = Vector4::from_column_slice(pos);
        let dir = Vector4::from_column_slice(
            &[dir[0], dir[1], 0.0, dir[2]]
        );

        let inv = self.code.get_matrix().try_inverse().unwrap();
        let new_pos = inv * pos;
        let new_dir = inv * dir;

        ([new_pos[0], new_pos[1], new_pos[2], new_pos[3]],
            [new_dir[0], new_dir[1], new_dir[3]])
    }

    pub fn get_code(&mut self) -> u32 { self.code.get_id() }
    // pub fn get_code_nonmut(&self) -> u32 {self.code.id.unwrap()}

    pub fn get_vbuf(&self) -> Option<&wgpu::Buffer> { self.vbuf.as_ref() }
    // pub fn get_ibuf(&self) -> Option<&wgpu::Buffer> { self.ibuf.as_ref() }
    pub fn get_pos(&self) -> Option<&wgpu::Buffer> { self.pos.as_ref() }

    pub fn get_size(&self) -> u32 { (self.indices.len() / 4) as u32 }

    pub fn set_vbuf(&mut self, buf: wgpu::Buffer) { self.vbuf = Some(buf); }
    pub fn set_ibuf(&mut self, buf: wgpu::Buffer) { self.ibuf = Some(buf); }
    pub fn set_pos(&mut self, buf: wgpu::Buffer) { self.pos = Some(buf); }

    pub fn get_bind_group(&self) -> Option<&wgpu::BindGroup>
        { self.bind_group.as_ref() }
    pub fn set_bind_group(&mut self, bg: wgpu::BindGroup)
        { self.bind_group = Some(bg); }
}
