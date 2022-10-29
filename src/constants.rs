pub const SCREEN_PIXELS: (u32, u32) = (640, 360);
pub const G_BUFFER_NUMS: u32 = 3;
pub const WORLD_SCALE: f32 = 4.5;
pub const NEAR_PLANE: f32 = 2500.0;
pub const FAR_PLANE: f32 = -2500.0;

pub const G_BUFFER_SIZE: u32 = 
      SCREEN_PIXELS.0
    * SCREEN_PIXELS.1
    * 4 // size of u32
    * G_BUFFER_NUMS; // data points per pixel
    //+ 8; // two extra numbers for depth info??
