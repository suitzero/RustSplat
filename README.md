# Rust Gaussian Splatting Renderer (wgpu with Mock Backend)

This project aims to build a Gaussian splatting renderer using Rust. It's architected around a `GraphicsBackend` trait to support multiple rendering backends, with an initial `MockRenderer` implementation to allow application development while actual GPU integration is pending/blocked.

## Architecture: `GraphicsBackend` Trait and `wgpu`

The core architecture revolves around a `GraphicsBackend` trait that abstracts rendering operations. This allows for:
-   **`MockRenderer`**: An initial implementation that logs rendering actions without actual GPU calls. This enables development and testing of higher-level application logic (scene management, camera, data structures) independently of specific graphics API issues.
-   **Future `WgpuRenderer`**: The plan is to implement this trait using `wgpu`. `wgpu` provides an abstraction over modern graphics APIs (Vulkan, Metal, DirectX 12, WebGPU), enabling cross-platform deployment on native systems and web browsers.

## Current Status: Application Logic Development with Mock Renderer

Development is proceeding with the `MockRenderer`:

-   **`GraphicsBackend` Trait**: Defined with methods for initialization, resize, update, and rendering.
-   **`MockRenderer`**: Implemented to log rendering operations and simulate behavior.
-   **Application Structure**:
    -   A `winit` event loop manages windowing and user interaction (placeholder input).
    -   An `App` struct encapsulates the main application state, owning:
        -   An instance of the `GraphicsBackend` (currently `MockRenderer`).
        -   `GaussianSplat` data structures and sample splats.
        -   A `Camera` struct with view/projection matrix calculations.
    -   The `App` struct processes updates and passes scene data (splats, camera) to the renderer.
-   **Data Structures**: `GaussianSplat` and `Camera` structs are defined.

**`wgpu` Integration BLOCKED**: The implementation of a `WgpuRenderer` is currently **BLOCKED** due to environment-specific issues with Rust crate resolution for `wgpu`. Standard versions of `wgpu` and their features are not being recognized correctly by Cargo in the build environment.

Once the `wgpu` environment issues are resolved, the `WgpuRenderer` will be implemented, and the application will transition to actual GPU-accelerated rendering.

## Goal

The ultimate goal remains to render 3D scenes using the Gaussian splatting technique, deployable across native platforms (using Vulkan via `wgpu`) and the web (using WebGPU via `wgpu`).

## Original Vulkan Setup (Pre-`wgpu`/Mock Pivot)

Previously, the project had successfully set up foundational Vulkan infrastructure directly using `ash`. This direct Vulkan code has been archived in previous commits in favor of the current `GraphicsBackend` trait and planned `wgpu` approach.
