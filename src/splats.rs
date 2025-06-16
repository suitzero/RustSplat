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

pub fn generate_procedural_splats(count_per_axis: u32, spacing: f32) -> Vec<GaussianSplat> {
    let mut splats = Vec::new();
    if count_per_axis == 0 { return splats; } // Avoid division by zero if count is 0

    let half_extent = (count_per_axis -1) as f32 * spacing / 2.0; // Center the grid properly

    for i in 0..count_per_axis {
        for j in 0..count_per_axis {
            for k in 0..count_per_axis {
                let x = i as f32 * spacing - half_extent;
                let y = j as f32 * spacing - half_extent;
                let z = k as f32 * spacing - half_extent;

                // Vary color based on position (simple gradient)
                // Ensure valid range for color components (0-255)
                let r_norm = (i as f32 / (count_per_axis.saturating_sub(1)).max(1) as f32).clamp(0.0, 1.0);
                let g_norm = (j as f32 / (count_per_axis.saturating_sub(1)).max(1) as f32).clamp(0.0, 1.0);
                let b_norm = (k as f32 / (count_per_axis.saturating_sub(1)).max(1) as f32).clamp(0.0, 1.0);

                let r = (r_norm * 255.0) as u8;
                let g = (g_norm * 255.0) as u8;
                let b = (b_norm * 255.0) as u8;

                // Vary scale slightly
                let base_scale = spacing / 3.0; // Make scale relative to spacing
                let scale_factor_variation = (i % 3 + j % 3 + k % 3) as f32 / 6.0; // Varies from 0.0 to 1.0
                let scale_factor = 0.75 + scale_factor_variation * 0.5; // Range 0.75 to 1.25
                let scale_val = base_scale * scale_factor;

                // Simple rotation pattern: rotate around Y axis based on i
                let angle_y = (i as f32 / count_per_axis as f32) * std::f32::consts::PI;
                let rotation = [ (angle_y / 2.0).cos(), 0.0, (angle_y / 2.0).sin(), 0.0 ];

                splats.push(GaussianSplat {
                    position: [x, y, z],
                    scale: [scale_val, scale_val, scale_val],
                    rotation,
                    color: [r, g, b, 255],
                    // Vary opacity slightly based on position, ensuring it's within [0,1]
                    opacity: (0.6 + (k as f32 / count_per_axis as f32) * 0.4).clamp(0.0,1.0),
                });
            }
        }
    }
    splats
}
