# Rust Gaussian Splatting Renderer (Vulkan)

This project is an attempt to build a Gaussian splatting renderer using Rust and the Vulkan API.

## Current Status (Initial Setup Phase)

The project has successfully set up the foundational Vulkan infrastructure:

-   **Rust Project**: Initialized with `ash` for Vulkan bindings and `winit` for windowing.
-   **Vulkan Core Initialization**:
    -   Vulkan instance and logical device created.
    -   Window and presentation surface established.
    -   Physical device selected, and queue families identified.
-   **Swapchain**: A functional swapchain is in place, along with image views for its images.
-   **Render Pass**: A basic render pass has been created, configured to clear the swapchain images and prepare them for presentation.

The application currently compiles and runs, displaying an empty window. The next immediate step is to implement the graphics pipeline to start rendering basic primitives.

## Goal

The ultimate goal is to render 3D scenes using the Gaussian splatting technique, a method for real-time rendering of photorealistic scenes from sparse 3D data.

## Next Steps (Post-Initial Setup)

1.  Create the initial Graphics Pipeline (simple triangle).
2.  Set up Framebuffers.
3.  Implement Command Buffers and basic render loop.
4.  Define data structures for Gaussian splats.
5.  Load/generate splat data.
6.  Update shaders and pipeline for splat rendering.
7.  Implement camera controls.
