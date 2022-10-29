use crate::game::tile::Tile;

use winit::window::Window;

use self::renderer::{ Renderer };
use crate::constants;

//mod texture;
pub mod renderer;

pub struct Screen {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    border: [f32; 2],

    renderer: Renderer,
}

impl Screen {
    pub async fn new(window: &Window) -> Screen {
        let mut limits = wgpu::Limits::default();
        limits.max_push_constant_size = 128;

        let size = window.inner_size();

        let goal_aspect_ratio = constants::SCREEN_PIXELS.0 as f32
            / constants::SCREEN_PIXELS.1 as f32;
        let actual_aspect_ratio = size.width as f32
            / size.height as f32;

        let mut border = [0f32; 2];
        if actual_aspect_ratio > goal_aspect_ratio{
            border[0] = (actual_aspect_ratio
                    - goal_aspect_ratio)
                / actual_aspect_ratio;
        } else {
            border[1] = ((1.0 / actual_aspect_ratio)
                    - (1.0 / goal_aspect_ratio))
                * actual_aspect_ratio;
        }

        // Stupid gpu setup
        let instance = wgpu::Instance::new(wgpu::Backends::all());
        let surface = unsafe { instance.create_surface(window) };
        let adapter = instance.request_adapter(
            &wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            },
        ).await.unwrap();
        let (device, queue) = adapter.request_device(
            &wgpu::DeviceDescriptor {
                features: wgpu::Features::PUSH_CONSTANTS |
                    wgpu::Features::CLEAR_TEXTURE,
                limits,
                label: None,
            },
            None,
        ).await.unwrap();
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface.get_supported_formats(&adapter)[0],
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
        };
        surface.configure(&device, &config);

        let renderer = Renderer::new(&device, &config);

        Self {
            surface,
            device,
            queue,
            config,
            size,
            border,

            renderer,
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);

            let goal_aspect_ratio = constants::SCREEN_PIXELS.0 as f32
                / constants::SCREEN_PIXELS.1 as f32;
            let actual_aspect_ratio = new_size.width as f32
                / new_size.height as f32;

            let mut border = [0f32; 2];
            if actual_aspect_ratio > goal_aspect_ratio{
                border[0] = (actual_aspect_ratio
                        - goal_aspect_ratio)
                    / actual_aspect_ratio;
            } else {
                border[1] = ((1.0 / actual_aspect_ratio)
                        - (1.0 / goal_aspect_ratio))
                    * actual_aspect_ratio;
            }
            self.border = border;
        }
    }

    pub fn render(
        &self,
        camera: &crate::game::camera::Camera,
        tiles: &mut [Tile]
    ) -> Result<(), wgpu::SurfaceError> {
        self.renderer.render(
            &self.device,
            &self.surface, 
            &self.queue, 
            camera, 
            tiles,
            &self.border
        )
    }

    pub fn updade_tile_pos(&self, tile: &mut Tile) {
        self.queue.write_buffer(
            tile.get_pos().unwrap(),
            0,
            bytemuck::cast_slice(tile.get_mat().as_slice())
        );
    }
}