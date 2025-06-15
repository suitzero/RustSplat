// src/camera.rs

// Simple 4x4 matrix, column-major for typical graphics APIs.
// (wgpu uses column-major by default for WGSL)
pub type Mat4 = [[f32; 4]; 4];

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: [f32; 3],
    pub target: [f32; 3],
    pub up: [f32; 3],
    pub fov_y_radians: f32, // Field of view in Y, in radians
    pub aspect_ratio: f32,
    pub near_plane: f32,
    pub far_plane: f32,
}

impl Camera {
    pub fn new(
        position: [f32; 3],
        target: [f32; 3],
        up: [f32; 3],
        fov_y_radians: f32,
        aspect_ratio: f32,
        near_plane: f32,
        far_plane: f32,
    ) -> Self {
        Self {
            position,
            target,
            up,
            fov_y_radians,
            aspect_ratio,
            near_plane,
            far_plane,
        }
    }

    pub fn identity_matrix() -> Mat4 {
        [
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ]
    }

    // Basic look_at_rh (right-handed) view matrix calculation
    // Transforms world space to view space (camera at origin, looking down -Z)
    pub fn calculate_view_matrix(&self) -> Mat4 {
        let pos = self.position;
        let target = self.target;
        let up_dir = self.up;

        // Camera's forward direction (f or z_axis)
        let f = normalize([target[0] - pos[0], target[1] - pos[1], target[2] - pos[2]]);
        // Camera's right direction (s or x_axis)
        let s = normalize(cross_product(f, up_dir));
        // Camera's actual up direction (u or y_axis)
        let u = cross_product(s, f); // s and f are orthonormal, so u is normalized

        // Column-major view matrix
        [
            [s[0], u[0], -f[0], 0.0],
            [s[1], u[1], -f[1], 0.0],
            [s[2], u[2], -f[2], 0.0],
            [
                -dot_product(s, pos),
                -dot_product(u, pos),
                dot_product(f, pos),
                1.0,
            ],
        ]
    }

    // Basic perspective_rh_zo (right-handed, depth 0-1) projection matrix
    pub fn calculate_projection_matrix(&self) -> Mat4 {
        let f_tan = 1.0 / (self.fov_y_radians / 2.0).tan();
        let ar = self.aspect_ratio;
        let n = self.near_plane;
        let f = self.far_plane;

        // Column-major projection matrix (for depth 0-1, typical for Vulkan/wgpu)
        // This maps depth to [0, 1] range.
        // X from [-w, w] to [-1, 1]
        // Y from [-h, h] to [-1, 1]
        // Z from [n, f] to [0, 1] (in view space, -Z is forward, so -n and -f map to 0 and 1)
        let mut p = [[0.0; 4]; 4];
        p[0][0] = f_tan / ar;
        p[1][1] = f_tan;
        p[2][2] = f / (f - n);     // Remaps Z to [0,1] after perspective divide
        p[2][3] = 1.0;              // Puts Z into W component for perspective divide
        p[3][2] = -(f * n) / (f - n); // Stores -N*F/(F-N)

        // Note: For a left-handed system or different Z range (-1 to 1), this matrix would change.
        // For WGSL/Naga, the NDC Z range is 0 to 1.
        // The projection matrix above correctly maps view space Z from [-n, -f] (assuming camera looks down -Z)
        // to NDC Z from [0, 1].
        // If using reversed Z (far at 0, near at 1) for better depth precision:
        // P[2][2] = n / (n - f);
        // P[3][2] = (f * n) / (n - f);
        // This usually accompanies a reversed depth test (Greater instead of Less).

        p
    }
}

// --- Minimal Vec3 Math Helpers ---
// These should ideally be replaced by a proper math library like glam or nalgebra
// if the environment/dependency constraints allow in the future.

fn normalize(v: [f32; 3]) -> [f32; 3] {
    let mag_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if mag_sq == 0.0 { return [0.0, 0.0, 0.0]; } // Avoid division by zero and sqrt(0)
    let mag = mag_sq.sqrt();
    [v[0] / mag, v[1] / mag, v[2] / mag]
}

fn cross_product(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

fn dot_product(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
