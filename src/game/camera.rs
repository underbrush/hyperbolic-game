pub struct Camera {
    pos: [f32; 4],
    dir: [f32; 3],
}

impl Camera {
    pub fn new() -> Camera {
        Self {pos: [0.0, 0.0, 0.0, 1.0], dir: [0.0, 1.0, 0.0]}
    }

    pub fn set_camera(&mut self, pos: [f32; 4], dir: [f32; 3]) {
        self.pos = pos;
        self.dir = dir;

        // v this oughta be moved into `player` when that becomes a thing
        self.make_perpendicular();
    }

    pub fn world_to_camera(&self) -> [[f32; 4]; 4] {
        let mut out = [[0.0; 4]; 4];
        let x2y2 = self.pos[0] * self.pos[0] + self.pos[1] * self.pos[1];
        if x2y2 == 0.0 {
            out[0][0] = self.dir[1];
            out[0][1] = self.dir[0];
            out[1][0] = -self.dir[0];
            out[1][1] = self.dir[1];
            out[2][2] = 1.0;
            out[3][3] = 1.0;
        } else {
            let xvyyvx = self.pos[0] * self.dir[1]
                - self.pos[1] * self.dir[0];

            out[0][0] = (self.pos[0] * self.pos[3] * xvyyvx
                + self.pos[1] * self.dir[2]) / x2y2;
            out[0][1] = (-self.pos[1] * xvyyvx
                + self.pos[0] * self.pos[3] * self.dir[2]) / x2y2;
            out[0][3] = -self.pos[0];

            out[1][0] = (self.pos[1] * self.pos[3] * xvyyvx
                - self.pos[0] * self.dir[2]) / x2y2;
            out[1][1] = (self.pos[0] * xvyyvx
                + self.pos[1] * self.pos[3] * self.dir[2]) / x2y2;
            out[1][3] = -self.pos[1];

            out[2][2] = 1.0;

            out[3][0] = -xvyyvx;
            out[3][1] = -self.dir[2];
            out[3][3] = self.pos[3];
        }
        out
    }

    pub fn camera_to_screen(&self) -> [[f32; 4]; 4] {
        let mut out = [[0.0; 4]; 4];
        out[0][0] = (6.0f32).sqrt() / 2.0;
        out[1][1] = -(2.0f32).sqrt() / 2.0;
        out[1][2] = -(6.0f32).sqrt() / 3.0;
        out[2][1] = -1.0;
        out[2][2] = (3.0f32).sqrt() / 2.0;
        out[3][3] = 1.0;
        out
    }

    pub fn get_position(&self) -> &[f32; 4] { &self.pos }

    pub fn get_direction(&self) -> &[f32; 3] { &self.dir }

    fn divergence(&self) -> (f32, f32, f32) {(
        self.dir[2] * self.pos[3]
      - self.dir[0] * self.pos[0]
      - self.dir[1] * self.pos[1],

        self.dir[2] * self.dir[2]
      - self.dir[0] * self.dir[0]
      - self.dir[1] * self.dir[1],

        self.pos[3] * self.pos[3]
      - self.pos[0] * self.pos[0]
      - self.pos[1] * self.pos[1]
    )}

    fn normalize(&mut self) {
        let p_mag = (self.pos[3] * self.pos[3]
            - self.pos[0] * self.pos[0]
            - self.pos[1] * self.pos[1]).abs();

        self.pos = [
            self.pos[0] / p_mag.sqrt(),
            self.pos[1] / p_mag.sqrt(),
            self.pos[2],
            self.pos[3] / p_mag.sqrt(),
        ];

        let v_mag = (self.dir[2] * self.dir[2]
            - self.dir[0] * self.dir[0]
            - self.dir[1] * self.dir[1]).abs();

        self.dir = [
            self.dir[0] / v_mag.sqrt(),
            self.dir[1] / v_mag.sqrt(),
            self.dir[2] / v_mag.sqrt(),
        ];
    }

    fn make_perpendicular(&mut self) {
        let (dot, v_div, p_div) = self.divergence();

        let thresh = 0.00005 * self.pos[3];
        if ((v_div + 1.0).abs() > thresh)
        || ((p_div - 1.0).abs() > thresh) {
            self.normalize();
        }
        if dot.abs() > thresh {
            let new_vect = [
                self.dir[0] - dot * self.pos[0],
                self.dir[1] - dot * self.pos[1],
                self.dir[2] - dot * self.pos[3]
            ];
            let mag = new_vect[2] * new_vect[2]
                - new_vect[0] * new_vect[0]
                - new_vect[1] * new_vect[1];
            self.dir = [
                new_vect[0] / (mag.abs().sqrt()),
                new_vect[1] / (mag.abs().sqrt()),
                new_vect[2] / (mag.abs().sqrt()),
            ];
        }
    }
}
