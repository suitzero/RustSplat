use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};

mod renderer;
mod camera; // Added mod declaration
mod splats; // Added mod declaration

use renderer::{GraphicsBackend, MockRenderer};
use camera::Camera; // Added import
use splats::{GaussianSplat, get_sample_splats}; // Added imports

struct App {
    renderer: Box<dyn GraphicsBackend>,
    camera: Camera,
    #[allow(dead_code)] // splats are not yet used for rendering in this mock setup
    splats: Vec<GaussianSplat>,
    last_frame_time: std::time::Instant,
    frame_delta_time: f32,
    // A simple accumulator for time to make animation smoother
    total_time_elapsed: f32,
}

impl App {
    fn new(renderer: Box<dyn GraphicsBackend>, initial_window_size: winit::dpi::PhysicalSize<u32>) -> Self {
        let aspect_ratio = if initial_window_size.height > 0 {
            initial_window_size.width as f32 / initial_window_size.height as f32
        } else {
            1.0 // Default aspect ratio if height is 0 to avoid division by zero
        };

        let camera = Camera::new(
            [0.0, 0.5, 2.5], // position (slightly further back)
            [0.0, 0.0, 0.0], // target
            [0.0, 1.0, 0.0], // up
            std::f32::consts::FRAC_PI_4, // fov_y_radians (45 degrees)
            aspect_ratio,
            0.1,  // near_plane
            100.0, // far_plane
        );

        let splats = get_sample_splats();
        log::info!("App initialized with {} sample splats.", splats.len());
        // Log camera initial state
        log::info!("Initial Camera: {:?}", camera);


        Self {
            renderer,
            camera,
            splats,
            last_frame_time: std::time::Instant::now(),
            frame_delta_time: 0.0,
            total_time_elapsed: 0.0,
        }
    }

    fn calculate_delta_time(&mut self) {
        let current_time = std::time::Instant::now();
        self.frame_delta_time = (current_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        self.total_time_elapsed += self.frame_delta_time;
    }

    #[allow(dead_code)] // update_input is not called yet
    fn update_input(&mut self /* winit::event::WindowEvent might be passed here later */) {
        // Placeholder for input handling
    }

    fn update_state(&mut self) {
        self.calculate_delta_time();
        self.renderer.update(); // Call renderer's update

        // Update application logic, animations, physics, etc.
        // Example: Simple camera orbit using total_time_elapsed for smoother animation
        self.camera.position[0] = (self.total_time_elapsed * 0.4).sin() * 2.5;
        self.camera.position[1] = 0.5 + (self.total_time_elapsed * 0.25).sin() * 0.5;
        self.camera.position[2] = (self.total_time_elapsed * 0.4).cos() * 2.5;

        // Periodically log camera state if needed for debugging animation
        // if (self.total_time_elapsed * 1000.0) as u64 % 1000 < 16 { // roughly every second
        //    log::info!("Camera position: {:?}", self.camera.position);
        // }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.resize(width, height);
            self.camera.aspect_ratio = width as f32 / height as f32;
            log::info!("App resized. New camera aspect_ratio: {}", self.camera.aspect_ratio);
        }
    }

    fn render(&mut self) -> Result<(), String> {
        // Pass the application's splats and camera to the renderer
        self.renderer.render(&self.splats, &self.camera)
    }
}

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::builder().format_timestamp_millis().init();

    log::info!("Application starting...");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Gaussian Splatting Renderer (Mock with App)")
        .with_inner_size(LogicalSize::new(1024, 768)) // Slightly larger default
        .build(&event_loop)
        .expect("Failed to create window");

    log::info!("Winit window created: {:?}", window.id());

    let initial_size = window.inner_size();
    let mock_renderer = MockRenderer::new(&window); // window is borrowed here
    let mut app = App::new(Box::new(mock_renderer), initial_size); // App takes ownership of the Boxed renderer

    // The window is owned by main's scope. App::new takes the renderer.
    // MockRenderer::new(&window) means the MockRenderer *borrows* the window for its setup.
    // This is fine as window outlives the MockRenderer's new() call.
    // If MockRenderer stored &'a Window, App would need to manage that lifetime.
    // But MockRenderer only stores window.id() and size, not a reference to window itself.
    // The GraphicsBackend trait's new(window: &Window) implies the backend might store it,
    // or use it just for setup. Our App struct does not store a reference to window,
    // it would get it via the event loop if needed for input processing.

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
                // Future: Pass relevant events to app.input(&event) if input handling is added to App
                match event {
                    WindowEvent::Resized(physical_size) => {
                        log::info!("Main: Window resized to {}x{}", physical_size.width, physical_size.height);
                        app.resize(physical_size.width, physical_size.height);
                    }
                    WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                        log::info!("Main: Window scale factor changed. New inner size: {}x{}", new_inner_size.width, new_inner_size.height);
                        app.resize(new_inner_size.width, new_inner_size.height);
                    }
                    WindowEvent::CloseRequested => {
                        log::info!("Main: Window close requested. Exiting.");
                        *control_flow = ControlFlow::Exit;
                    }
                    _ => (), // Other window events
                }
            }
            Event::RedrawRequested(event_window_id) if event_window_id == window.id() => {
                app.update_state(); // Update app logic (which now also calls renderer.update())
                match app.render() {
                    Ok(_) => {}
                    Err(e) => {
                        log::error!("App render error: {}", e);
                    }
                }
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            Event::LoopDestroyed => {
                log::info!("Main: Event loop destroyed.");
            }
            _ => (),
        }
    });
}
