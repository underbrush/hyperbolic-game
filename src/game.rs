use winit::{ event::*, window::Window };
use self::screen::Screen;
use self::camera::Camera;
use self::tile::Tile;
use self::tile::permutation::GroupElt;

mod screen;
mod camera;
mod tile;

pub struct Game{
    screen: Screen,
    camera: Camera,
    tiles: Vec<Tile>,
    codes: Vec<u32>,

    l_pressed: bool, // MAKE LIKE A HASH TABLE FOR THIS
    r_pressed: bool,
    u_pressed: bool,
    d_pressed: bool,
}

impl Game {
    pub async fn new(window: &Window) -> Game {
        let screen = Screen::new(window).await;
        let camera = Camera::new();

        let mut out = Self {
            screen,
            camera,
            tiles: Vec::<Tile>::new(),
            codes: Vec::<u32>::new(),

            l_pressed: false,
            r_pressed: false,
            u_pressed: false,
            d_pressed: false,
        };

        out.tiles.push(Tile::new(tile::permutation::IDENTITY));
        out.codes.push(0_u32);

        out
    }

    pub fn update(&mut self) {
        /* #region STUPDI MOVEMENT GARBAGE (FIX) (STUPID) */
        let distance = 0.005f32;

        let direction = self.camera.get_direction();
        let position = self.camera.get_position();
        if self.u_pressed {
            let new_pos = [
                position[0] * distance.cosh() + direction[0] * distance.sinh(),
                position[1] * distance.cosh() + direction[1] * distance.sinh(),
                position[2],
                position[3] * distance.cosh() + direction[2] * distance.sinh()];
            let new_dir = [
                position[0] * distance.sinh() + direction[0] * distance.cosh(),
                position[1] * distance.sinh() + direction[1] * distance.cosh(),
                position[3] * distance.sinh() + direction[2] * distance.cosh()];
            self.camera.set_camera(new_pos, new_dir);
        } else if self.d_pressed {
            let new_pos = [
                position[0] * distance.cosh() + -direction[0] * distance.sinh(),
                position[1] * distance.cosh() + -direction[1] * distance.sinh(),
                position[2],
                position[3] * distance.cosh() + -direction[2] * distance.sinh()];
            let new_dir = [
                -position[0] * distance.sinh() + direction[0] * distance.cosh(),
                -position[1] * distance.sinh() + direction[1] * distance.cosh(),
                -position[3] * distance.sinh() + direction[2] * distance.cosh()];
            self.camera.set_camera(new_pos, new_dir);
        }

        let position = self.camera.get_position();
        let old_direction = self.camera.get_direction();
        let direction = [
            position[1] * old_direction[2] - position[3] * old_direction[1],
            position[3] * old_direction[0] - position[0] * old_direction[2],
            position[1] * old_direction[0] - position[0] * old_direction[1],
        ];

        if self.l_pressed {
            let new_pos = [
                position[0] * distance.cosh() + direction[0] * distance.sinh(),
                position[1] * distance.cosh() + direction[1] * distance.sinh(),
                position[2],
                position[3] * distance.cosh() + direction[2] * distance.sinh()];
            self.camera.set_camera(new_pos, *old_direction);
        } else if self.r_pressed {
            let new_pos = [
                position[0] * distance.cosh() + -direction[0] * distance.sinh(),
                position[1] * distance.cosh() + -direction[1] * distance.sinh(),
                position[2],
                position[3] * distance.cosh() + -direction[2] * distance.sinh()];
            self.camera.set_camera(new_pos, *old_direction);
        }
        /* #endregion */

        /* #region TILE LOADING AND UNLOADING */
        let mut add_tiles = Vec::<Tile>::new();
        let mut remove_codes = Vec::<u32>::new();
        for tile in &mut self.tiles {
            let dist = tile.distance_from(self.camera.get_position());
            if dist < 1.3 {
                let (a, b) = tile.update_neighbors(
                    self.camera.get_position(),
                    &self.codes
                );

                add_tiles.extend(a);
                self.codes.extend(b);
            } else if dist > 2.8 {
                remove_codes.push(tile.get_code());
            }
        }
        self.tiles.extend(add_tiles);
        let l = self.codes.len();
        let mut dec = 0;
        for i in 0..l {
            if remove_codes.contains(&self.tiles[i - dec].get_code()) {
                self.tiles.swap_remove(i - dec);
                self.codes.swap_remove(i - dec);
                dec += 1;
            }
        }
        /* #endregion */
        
        /* #region MAP RE-CENTERING */

        let dist = self.camera.get_position()[3].acosh();
        if dist > 3.7 {
            let mut moved = Vec::<Tile>::new();
            let mut to_move = Vec::<GroupElt>::new();
            let mut min_dist = 4.5;
            let mut closest_tile = &self.tiles[0];

            for tile in &self.tiles {
                let new_dist = tile.distance_from(
                    self.camera.get_position()
                );
                if new_dist < min_dist {
                    min_dist = new_dist;
                    closest_tile = tile;
                }
            }

            let new_camera = closest_tile.move_against(
                self.camera.get_position(),
                self.camera.get_direction()
            );
            self.camera.set_camera(new_camera.0, new_camera.1);

            println!("{:?}", closest_tile.centered_code().get_id());

            to_move.push(closest_tile.centered_code());

            while to_move.len() > 0 && self.tiles.len() > 0 {
                let mut next = to_move.remove(to_move.len() - 1);

                for i in 0..self.tiles.len() {
                    if self.tiles[i].get_code() == next.get_id() {
                        let mut moved_tile = self.tiles.swap_remove(i);
                        to_move.extend(moved_tile.move_to(next));
                        self.screen.updade_tile_pos(&mut moved_tile);
                        moved.push(moved_tile);
                        break;
                    }
                }
            }

            self.tiles = moved;
        }

        /* #endregion */
    }

    pub fn handle_event(&mut self, event: &Event<()>, window: &Window) -> bool {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if *window_id == window.id() => if !self.input(event) {
                match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                state: ElementState::Pressed,
                                virtual_keycode: Some(VirtualKeyCode::Escape),
                                ..
                            },
                        ..
                    } => return true,
                    WindowEvent::Resized(physical_size) => {
                        self.screen.resize(*physical_size);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        // new_inner_size is &&mut so we have to dereference it twice
                        self.screen.resize(**new_inner_size);
                    }
                    _ => {}
                }
            },
            _ => {}
        }
        return false
    }

    fn input(&mut self, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::KeyboardInput {
                input: KeyboardInput {
                    state,
                    virtual_keycode: Some(keycode),
                    ..
                },
                ..
            } => {
                let is_pressed = *state == ElementState::Pressed;
                match keycode {
                    VirtualKeyCode::W | VirtualKeyCode::Up => {
                        self.u_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::A | VirtualKeyCode::Left => {
                        self.l_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::S | VirtualKeyCode::Down => {
                        self.d_pressed = is_pressed;
                        true
                    }
                    VirtualKeyCode::D | VirtualKeyCode::Right => {
                        self.r_pressed = is_pressed;
                        true
                    }
                    _ => false,
                }
            },
            _ => false
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let len = self.tiles.len();
        self.screen.render(
            &self.camera, 
            &mut self.tiles[0..len]
        )
    }

    pub fn reconfigure(&mut self) {
        self.screen.resize(self.screen.size)
    }

}