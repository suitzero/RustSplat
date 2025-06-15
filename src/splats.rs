// src/splats.rs

// Using simple arrays for now. Could use a math library like glam or nalgebra later
// if the dependency environment allows.
#[derive(Debug, Clone)]
pub struct GaussianSplat {
    pub position: [f32; 3],    // x, y, z
    pub scale: [f32; 3],       // scale_x, scale_y, scale_z (log-space representation often used)
    pub rotation: [f32; 4],    // q_w, q_x, q_y, q_z (quaternion for rotation)
    pub color: [u8; 4],        // R, G, B, A (normalized u8, or f32 for HDR)
    pub opacity: f32,          // Opacity (often combined with color's alpha or separate)
    // pub sh_coefficients: Vec<[f32; 3]>, // Spherical Harmonics coefficients for view-dependent color (optional, more advanced)
}

impl GaussianSplat {
    pub fn new_default(_id: u32) -> Self { // id parameter is not used yet, but kept for future consistency
        // Create a default splat for testing
        Self {
            position: [0.0, 0.0, 0.0],
            // Default scale to something small but visible
            scale: [0.1, 0.1, 0.1],
            // Default rotation: identity quaternion (w=1, x=0, y=0, z=0)
            rotation: [1.0, 0.0, 0.0, 0.0],
            // Default color: white
            color: [255, 255, 255, 255],
            opacity: 1.0,
            // sh_coefficients: Vec::new(), // Empty for now
        }
    }
}

// Example function to create a few sample splats (can be expanded)
pub fn get_sample_splats() -> Vec<GaussianSplat> {
    vec![
        GaussianSplat {
            position: [0.0, 0.0, 0.0],
            scale: [0.2, 0.2, 0.2],
            rotation: [1.0, 0.0, 0.0, 0.0], // Identity quaternion
            color: [255, 0, 0, 255], // Red
            opacity: 1.0,
        },
        GaussianSplat {
            position: [0.5, 0.0, 0.0],
            scale: [0.1, 0.1, 0.1],
            rotation: [1.0, 0.0, 0.0, 0.0],
            color: [0, 255, 0, 255], // Green
            opacity: 0.8,
        },
        GaussianSplat {
            position: [0.0, 0.5, 0.0],
            scale: [0.15, 0.15, 0.05],
            rotation: [0.7071, 0.0, 0.7071, 0.0], // ~90 deg rotation around Y
            color: [0, 0, 255, 200], // Blue, slightly transparent
            opacity: 0.78, // opacity = color_alpha / 255.0 (if color is u8)
        },
    ]
}
