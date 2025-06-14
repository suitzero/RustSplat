# Rust Gaussian Splatting Renderer (wgpu)

This project aims to build a Gaussian splatting renderer using Rust, leveraging the `wgpu` library for cross-platform graphics.

## Architecture: `wgpu`

The project has transitioned to using `wgpu`. This decision offers several advantages:

-   **Cross-Platform Compatibility**: `wgpu` provides an abstraction over modern graphics APIs. This means the same renderer codebase can target:
    -   **Native Platforms**: Vulkan (on Linux/Windows), Metal (on macOS), DirectX 12 (on Windows).
    -   **Web Browsers**: WebGPU, by compiling the Rust code to WebAssembly.
-   **Modern Graphics API**: `wgpu` exposes a modern, explicit graphics API conceptually similar to Vulkan, Metal, and DX12.
-   **Rust Ecosystem**: It integrates well with the Rust ecosystem and `winit` for windowing.

## Current Status: Initial `wgpu` Setup (Blocked)

The project is currently in the initial phase of setting up `wgpu`.

-   **Dependencies**: `Cargo.toml` has been updated to include `wgpu` and necessary helper crates. The previous `ash` (direct Vulkan) dependencies have been removed.
-   **Core `wgpu` Structure**: Work has begun on defining a core `State` object to manage `wgpu` components (Instance, Adapter, Device, Queue, Surface, SurfaceConfiguration).

**Implementation is currently BLOCKED due to environment-specific issues with Rust crate resolution for `wgpu`. Standard versions of `wgpu` and their features are not being recognized correctly by Cargo in the build environment. Further progress on `wgpu` implementation is contingent on resolving these external environment problems.**

Once the environment issues are resolved, the next steps will involve:
1.  Successfully initializing the core `wgpu` components.
2.  Integrating `wgpu` with the `winit` event loop.
3.  Implementing a basic render pipeline using `wgpu` and WGSL shaders.

## Goal

The ultimate goal remains to render 3D scenes using the Gaussian splatting technique, now with the flexibility and reach provided by `wgpu`.

## Original Vulkan Setup (Pre-`wgpu` Pivot)

Previously, the project had successfully set up foundational Vulkan infrastructure directly using `ash`:
-   Vulkan instance, logical device, window/surface, swapchain, image views, and a basic render pass were established.
This direct Vulkan code has been archived in previous commits in favor of the `wgpu` approach.
