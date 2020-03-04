extern crate sdl2;
extern crate gl;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct Pixel
{
    pub b: f32,
    pub g: f32,
    pub r: f32,
    pub a: f32,
}

macro_rules! TRY_D
{
    ($e: expr) =>
    {
        match $e
        {
            Ok(_) => (),
            Err(error) => println!("{}", error),
        }
    }
}

pub struct RenderTexture
{
    pub pixels: Vec<Pixel>,
    pub width: u32,
    pub height: u32,
    pub id: u32,
}

#[allow(dead_code)] // The user may not need to use everything.
pub struct Application
{
    pub sdl_context: sdl2::Sdl,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub window: sdl2::video::Window,
    pub back_buffer: RenderTexture,
}

impl Copy for Pixel {}

impl Clone for Pixel
{
    fn clone(&self) -> Self {
        *self
    }
}

pub trait Renderer
{
    fn init(&mut self, app: &mut Application);
    fn render(&mut self, app: &mut Application);
}

pub struct AppBuilder
{
    title: String,
    width: u32,
    height: u32,
    running: bool,
}

impl AppBuilder
{
    pub fn new(title: &str, width: u32, height: u32) -> AppBuilder
    {
        AppBuilder
        {
            title: title.to_owned(),
            width,
            height,
            running: false,
        }
    }

    pub fn start(&mut self, renderer_trait: &mut dyn Renderer)
    {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let window = video_subsystem
            .window(&self.title, self.width, self.height)
            .hidden()
            .opengl()
            .resizable()
            .build()
            .unwrap();

        // Init OpenGL
        let _gl_context = window.gl_create_context().unwrap();
        gl::load_with(|s| video_subsystem.gl_get_proc_address(s) as *const std::os::raw::c_void);

        // Disable VSync
        TRY_D!(video_subsystem.gl_set_swap_interval(0));

        let mut event_pump = sdl_context.event_pump().unwrap();

        /*
        Create Upload Frame Buffer
        */
        let back_buffer_texture = self.create_texture(self.width, self.height);
        let mut upload_frame_buffer: u32 = 0;
        self.create_fb(&mut upload_frame_buffer);

        let mut app = Application { sdl_context, video_subsystem, window, back_buffer: back_buffer_texture };

        renderer_trait.init(&mut app);

        self.show(&mut app);

        self.running = true;
        while self.running
        {
            self.parse_events(&mut event_pump);

            renderer_trait.render(&mut app);

            self.update_back_buffer(&app.back_buffer);
            self.copy_back_to_front(&app.back_buffer);

            app.window.gl_swap_window();
        }
    }

    fn copy_back_to_front(&self, texture: &RenderTexture)
    {
        unsafe
        {
            // Bind source texture
            gl::BindFramebuffer(gl::READ_FRAMEBUFFER, texture.id);
            gl::FramebufferTexture(gl::READ_FRAMEBUFFER, gl::COLOR_ATTACHMENT0, texture.id, 0);

            // Bind current back buffer.
            gl::BindFramebuffer(gl::DRAW_FRAMEBUFFER, gl::FRONT);
            gl::FramebufferTexture(gl::DRAW_FRAMEBUFFER, gl::COLOR_ATTACHMENT1, gl::FRONT, 0);

            // Copy texture to back buffer.
            gl::BlitFramebuffer(0, 0, texture.width as i32, texture.height as i32,
                                0, 0, self.width as i32, self.height as i32,
                                gl::COLOR_BUFFER_BIT, gl::NEAREST);

            // Unbind the color attachments.
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT1, 0, 0);
            gl::FramebufferTexture(gl::FRAMEBUFFER, gl::COLOR_ATTACHMENT0, 0, 0);
        }
    }

    fn update_back_buffer(&self, texture: &RenderTexture)
    {
        unsafe
        {
            gl::BindTexture(gl::TEXTURE_2D, texture.id);
            gl::TexSubImage2D(gl::TEXTURE_2D, 0, 0, 0, texture.width as i32, texture.height as i32,
                              gl::BGRA, gl::FLOAT, texture.pixels.as_ptr() as *const std::os::raw::c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }
    }

    fn create_fb(&self, id: &mut u32)
    {
        unsafe
        {
            gl::GenFramebuffers(1, id);
        }
    }

    fn create_texture(&self, width: u32, height: u32) -> RenderTexture
    {
        let mut id: u32 = 0;
        let pixels = vec![Pixel{r: 0f32, g: 0f32, b: 0f32, a: 0f32}; (width * height) as usize];

        unsafe
        {
            gl::GenTextures(1, &mut id);

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, id);
            gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, self.width as i32, self.height as i32,
                           0, gl::BGRA, gl::FLOAT, pixels.as_ptr() as *const std::os::raw::c_void);
            gl::BindTexture(gl::TEXTURE_2D, 0);
        }

        return RenderTexture{ pixels, width, height, id };
    }

    fn parse_events(&mut self, event_pump: &mut sdl2::EventPump)
    {
        use sdl2::event::WindowEvent;

        for event in event_pump.poll_iter()
        {
            match event
            {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } =>
                {
                    self.running = false;
                },
                Event::Window { win_event: WindowEvent::Resized(width, height), ..} =>
                {
                    self.resize_event(width, height);
                },
                _ => {}
            }
        }
    }

    fn resize_event(&mut self, width: i32, height: i32)
    {
        if width > 0 && height > 0
        {
            self.width = width as u32;
            self.height = height as u32;
        }
    }

    fn clear(&self)
    {
        unsafe
        {
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::ClearColor(0.3, 0.3, 0.5, 1.0);
        }
    }

    fn show(&self, app: &mut Application)
    {
        self.clear();
        app.window.gl_swap_window();
        app.window.show();
    }
}
