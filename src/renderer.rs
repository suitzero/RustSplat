use winit::window::{Window, WindowId};

// Import the actual structs from other modules
use crate::splats::GaussianSplat;
use crate::camera::Camera;

// Placeholder SplatData and CameraData structs are now removed.

pub trait GraphicsBackend {
    /// Creates a new backend instance.
    /// The window is passed as a reference and its lifetime should be managed by the caller (e.g., main application loop).
    fn new(window: &Window) -> Self where Self: Sized;

    /// Called when the window is resized.
    fn resize(&mut self, width: u32, height: u32);

    /// Called before `render`, for any state updates needed per frame.
    fn update(&mut self);

    /// Renders a frame.
    /// Now takes scene data (splats and camera).
    fn render(&mut self, splats: &[GaussianSplat], camera: &Camera) -> Result<(), String>;

    // Example methods for managing resources (will be expanded)
    // fn load_splats(&mut self, splats: &[GaussianSplat]) -> Result<u32, String>; // Returns a buffer ID
    // fn update_camera(&mut self, camera: &Camera) -> Result<(), String>;
}

// MockRenderer implementation
pub struct MockRenderer {
    window_id: WindowId,
    width: u32,
    height: u32,
    frame_count: u64, // For varying log messages
}

impl GraphicsBackend for MockRenderer {
    fn new(window: &Window) -> Self {
        let initial_size = window.inner_size();
        log::info!( // Changed from println!
            "MockRenderer: Initialized for window {:?} with size {}x{}",
            window.id(),
            initial_size.width,
            initial_size.height
        );
        Self {
            window_id: window.id(),
            width: initial_size.width,
            height: initial_size.height,
            frame_count: 0,
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        log::info!( // Changed from println!
            "MockRenderer: Resized to {}x{}. (Window ID: {:?})",
            self.width, self.height, self.window_id
        );
    }

    fn update(&mut self) {
        self.frame_count += 1;
        if self.frame_count % 60 == 0 { // Log every 60 updates
            log::info!( // Changed from println!
                "MockRenderer: Update cycle {} for window {:?}. Current size: {}x{}",
                self.frame_count, self.window_id, self.width, self.height
            );
        }
    }

    // Updated render method:
    fn render(&mut self, splats: &[GaussianSplat], camera: &Camera) -> Result<(), String> {
        log::info!(
            "MockRenderer: Rendering frame {} for window {:?}. Target size: {}x{}. Received {} splats. Camera at: [{:.2}, {:.2}, {:.2}]",
            self.frame_count,
            self.window_id,
            self.width,
            self.height,
            splats.len(),
            camera.position[0],
            camera.position[1],
            camera.position[2]
        );

        if !splats.is_empty() {
            // Log first splat as debug detail, as it can be verbose
            log::debug!("MockRenderer: First splat details: {:?}", splats[0]);
        }
        // Log matrices as debug, as they are very verbose
        log::debug!("MockRenderer: Camera view matrix: {:?}", camera.calculate_view_matrix());
        log::debug!("MockRenderer: Camera projection matrix: {:?}", camera.calculate_projection_matrix());

        Ok(())
    }
}
