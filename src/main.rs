use winit::{
    event::{Event, WindowEvent, VirtualKeyCode, ElementState}, // Added VirtualKeyCode, ElementState
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};

mod renderer;
mod camera;
mod splats;

use renderer::{GraphicsBackend, MockRenderer};
use camera::Camera;
use splats::{GaussianSplat, get_sample_splats, generate_procedural_splats};

// --- Vec3 Math Helpers ---
// These are simple implementations. A proper math library (glam, nalgebra) would be preferred
// if dependencies were not an issue for the environment.
#[inline]
fn normalize_vec3(v: [f32; 3]) -> [f32; 3] {
    let mag_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if mag_sq < f32::EPSILON * f32::EPSILON { // Check against a small epsilon
        return [0.0, 0.0, 0.0];
    }
    let mag = mag_sq.sqrt();
    [v[0] / mag, v[1] / mag, v[2] / mag]
}

#[inline]
fn cross_product_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

#[inline]
fn add_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

#[inline]
fn sub_vec3(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

#[inline]
fn scale_vec3(v: [f32; 3], s: f32) -> [f32; 3] {
    [v[0] * s, v[1] * s, v[2] * s]
}
// --- End of Vec3 Math Helpers ---

#[derive(Debug, Default)]
struct InputState {
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
    is_up_pressed: bool,
    is_down_pressed: bool,
    is_yaw_left_pressed: bool,
    is_yaw_right_pressed: bool,
    is_pitch_up_pressed: bool,
    is_pitch_down_pressed: bool,
}

struct App {
    renderer: Box<dyn GraphicsBackend>,
    camera: Camera,
    #[allow(dead_code)] // splats are not yet used for rendering in this mock setup
    splats: Vec<GaussianSplat>,
    last_frame_time: std::time::Instant,
    frame_delta_time: f32,
    total_time_elapsed: f32,
    input_state: InputState,
}

impl App {
    fn new(renderer: Box<dyn GraphicsBackend>, initial_window_size: winit::dpi::PhysicalSize<u32>) -> Self {
        let aspect_ratio = if initial_window_size.height > 0 {
            initial_window_size.width as f32 / initial_window_size.height as f32
        } else {
            1.0
        };

        let camera = Camera::new(
            [0.0, 0.5, 3.0], // position (start slightly further back)
            [0.0, 0.0, 0.0], // target
            [0.0, 1.0, 0.0], // up
            std::f32::consts::FRAC_PI_4,
            aspect_ratio,
            0.1,
            100.0,
        );

        let splats = generate_procedural_splats(5, 0.5); // New: 5x5x5 grid, 0.5 spacing
        log::info!("App initialized with {} procedurally generated splats.", splats.len());
        log::info!("Initial Camera: {:?}", camera);

        Self {
            renderer,
            camera,
            splats,
            last_frame_time: std::time::Instant::now(),
            frame_delta_time: 0.0,
            total_time_elapsed: 0.0,
            input_state: InputState::default(),
        }
    }

    fn calculate_delta_time(&mut self) {
        let current_time = std::time::Instant::now();
        self.frame_delta_time = (current_time - self.last_frame_time).as_secs_f32();
        self.last_frame_time = current_time;
        self.total_time_elapsed += self.frame_delta_time;
    }

    fn process_keyboard_input(&mut self, keycode: Option<VirtualKeyCode>, state: ElementState) {
        let pressed = state == ElementState::Pressed;
        match keycode {
            Some(VirtualKeyCode::W) | Some(VirtualKeyCode::Up) => self.input_state.is_forward_pressed = pressed,
            Some(VirtualKeyCode::S) | Some(VirtualKeyCode::Down) => self.input_state.is_backward_pressed = pressed,
            Some(VirtualKeyCode::A) | Some(VirtualKeyCode::Left) => self.input_state.is_left_pressed = pressed,
            Some(VirtualKeyCode::D) | Some(VirtualKeyCode::Right) => self.input_state.is_right_pressed = pressed,
            Some(VirtualKeyCode::Space) | Some(VirtualKeyCode::R) => self.input_state.is_up_pressed = pressed,
            Some(VirtualKeyCode::LShift) | Some(VirtualKeyCode::LControl) | Some(VirtualKeyCode::F) => self.input_state.is_down_pressed = pressed,
            Some(VirtualKeyCode::Q) => self.input_state.is_yaw_left_pressed = pressed,
            Some(VirtualKeyCode::E) => self.input_state.is_yaw_right_pressed = pressed,
            Some(VirtualKeyCode::T) => self.input_state.is_pitch_up_pressed = pressed,
            Some(VirtualKeyCode::G) => self.input_state.is_pitch_down_pressed = pressed,
            _ => {}
        }
    }

    fn update_state(&mut self) {
        self.calculate_delta_time();
        self.renderer.update();

        let camera_speed = 2.0 * self.frame_delta_time; // Adjusted speed
        let rotation_speed = 1.0 * self.frame_delta_time; // Adjusted speed

        let mut new_pos = self.camera.position;
        let mut new_target = self.camera.target;

        // Forward is from camera eye to target
        let forward_vec = normalize_vec3(sub_vec3(self.camera.target, self.camera.position));
        // Right is perpendicular to forward and world up (or camera up if implementing roll)
        let right_vec = normalize_vec3(cross_product_vec3(forward_vec, self.camera.up));
        // Actual camera up vector (if implementing roll, this would need to be updated by rotations)
        // let up_vec = self.camera.up; // For now, assume up is fixed relative to camera unless pitched. - currently unused
                                     // For simple FPS-style up/down, use world_up_vec or camera_up_vec.
        let world_up_vec = [0.0, 1.0, 0.0];


        if self.input_state.is_forward_pressed {
            new_pos = add_vec3(new_pos, scale_vec3(forward_vec, camera_speed));
            // Target also moves with the camera to maintain direction
            new_target = add_vec3(new_target, scale_vec3(forward_vec, camera_speed));
        }
        if self.input_state.is_backward_pressed {
            new_pos = sub_vec3(new_pos, scale_vec3(forward_vec, camera_speed));
            new_target = sub_vec3(new_target, scale_vec3(forward_vec, camera_speed));
        }
        if self.input_state.is_left_pressed {
            new_pos = sub_vec3(new_pos, scale_vec3(right_vec, camera_speed));
            new_target = sub_vec3(new_target, scale_vec3(right_vec, camera_speed));
        }
        if self.input_state.is_right_pressed {
            new_pos = add_vec3(new_pos, scale_vec3(right_vec, camera_speed));
            new_target = sub_vec3(new_target, scale_vec3(right_vec, camera_speed));
        }
        if self.input_state.is_up_pressed { // Move along world up
            new_pos = add_vec3(new_pos, scale_vec3(world_up_vec, camera_speed));
            new_target = add_vec3(new_target, scale_vec3(world_up_vec, camera_speed));
        }
        if self.input_state.is_down_pressed { // Move along world down
            new_pos = sub_vec3(new_pos, scale_vec3(world_up_vec, camera_speed));
            new_target = sub_vec3(new_target, scale_vec3(world_up_vec, camera_speed));
        }

        self.camera.position = new_pos;
        // For rotations, calculate the direction vector from new_pos to new_target
        let mut direction_vec = sub_vec3(new_target, new_pos);

        // Yaw (around camera's current up vector - self.camera.up)
        let yaw_angle = if self.input_state.is_yaw_left_pressed { rotation_speed }
                        else if self.input_state.is_yaw_right_pressed { -rotation_speed }
                        else { 0.0 };

        if yaw_angle.abs() > f32::EPSILON {
            let (s, c) = (yaw_angle.sin(), yaw_angle.cos());
            // Rotate around self.camera.up (which is [0,1,0] initially)
            // This is a simplified yaw assuming up is Y. For a generic camera_up, use Rodrigues' rotation formula or quaternions.
            let x = direction_vec[0];
            let z = direction_vec[2];
            direction_vec[0] = x * c - z * s;
            direction_vec[2] = x * s + z * c;
        }

        // Pitch (around camera's right vector)
        let pitch_angle = if self.input_state.is_pitch_up_pressed { rotation_speed }
                          else if self.input_state.is_pitch_down_pressed { -rotation_speed }
                          else { 0.0 };

        if pitch_angle.abs() > f32::EPSILON {
            let (s, c) = (pitch_angle.sin(), pitch_angle.cos());
            // let current_right_vec = normalize_vec3(cross_product_vec3(direction_vec, self.camera.up)); // Recalculate right based on current direction - currently unused

            // Rotate direction_vec around current_right_vec (Rodrigues' rotation formula simplified)
            // This is more complex than simple Y adjustment if we want to avoid gimbal lock / maintain up vector correctly.
            // For a simpler pitch:
            let y = direction_vec[1];
            let planar_dist = (direction_vec[0]*direction_vec[0] + direction_vec[2]*direction_vec[2]).sqrt();
            direction_vec[1] = y * c - planar_dist * s;
            // And scale the planar components
            let new_planar_dist = y * s + planar_dist * c;
            if planar_dist > f32::EPSILON { // Avoid division by zero if looking straight up/down
                 let scale_factor = new_planar_dist / planar_dist;
                 direction_vec[0] *= scale_factor;
                 direction_vec[2] *= scale_factor;
            } else { // If looking straight up/down, pitch means moving along original up/down
                 direction_vec[0] = 0.0; // Keep it aligned, or handle based on specific desired behavior
                 direction_vec[2] = 0.0;
            }
            // Normalize to maintain constant distance to target focus point if desired, or let target move freely.
            // direction_vec = normalize_vec3(direction_vec); // Optional: keep target at fixed distance

            // Also need to rotate the camera's `up` vector to avoid issues when looking straight up/down.
            // If not, `cross_product_vec3(forward_vec, self.camera.up)` for `right_vec` can become unstable.
            // A full quaternion or rotation matrix approach for camera orientation is better for complex rotations.
            // For now, let's update self.camera.up based on the new direction and a fixed world right/left.
            // The up vector used for cross_product_vec3 to define right_vec inside calculate_view_matrix
            // is self.camera.up. So, if we change self.camera.up here, it affects the next frame's controls.
            let new_forward = normalize_vec3(direction_vec);
            let global_right = [1.0, 0.0, 0.0]; // Simplification: assumes world right for camera 'up' calculation
            let new_camera_up = normalize_vec3(cross_product_vec3(global_right, new_forward));
            // Only update if the new_camera_up is not zero (which can happen if new_forward is parallel to global_right)
            if (new_camera_up[0]*new_camera_up[0] + new_camera_up[1]*new_camera_up[1] + new_camera_up[2]*new_camera_up[2]) > f32::EPSILON * f32::EPSILON {
                 self.camera.up = new_camera_up;
            }
        }

        self.camera.target = add_vec3(new_pos, direction_vec);
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.renderer.resize(width, height);
            self.camera.aspect_ratio = width as f32 / height as f32;
            log::info!("App resized. New camera aspect_ratio: {}", self.camera.aspect_ratio);
        }
    }

    fn render(&mut self) -> Result<(), String> {
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
        .with_inner_size(LogicalSize::new(1024, 768))
        .build(&event_loop)
        .expect("Failed to create window");

    log::info!("Winit window created: {:?}", window.id());

    let initial_size = window.inner_size();
    let mock_renderer = MockRenderer::new(&window);
    let mut app = App::new(Box::new(mock_renderer), initial_size);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent { window_id, event } if window_id == window.id() => {
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
                    WindowEvent::KeyboardInput {
                        input: winit::event::KeyboardInput {
                            state,
                            virtual_keycode,
                            ..
                        },
                        ..
                    } => {
                        app.process_keyboard_input(virtual_keycode, state);
                    }
                    _ => (),
                }
            }
            Event::RedrawRequested(event_window_id) if event_window_id == window.id() => {
                app.update_state();
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
