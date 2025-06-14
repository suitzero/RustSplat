use ash::{vk, Entry, Instance, Device};
use ash::extensions::khr::Surface as AshSurface; // Explicitly alias for clarity
use ash::extensions::khr::Swapchain as AshSwapchain; // For the swapchain loader
use std::ffi::{CString, CStr};
use std::os::raw::c_char;

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use raw_window_handle::{HasRawWindowHandle, HasRawDisplayHandle};

fn main() {
    println!("Application starting...");

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Gaussian Splatting Renderer")
        .with_inner_size(LogicalSize::new(800, 600))
        .build(&event_loop)
        .expect("Failed to create window.");
    println!("Winit window created.");

    let app_name = CString::new("Gaussian Splatting Renderer").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let entry = match unsafe { Entry::load() } {
        Ok(entry) => entry,
        Err(err) => {
            eprintln!("Failed to load Vulkan entry point: {}", err);
            return;
        }
    };
    println!("Vulkan entry loaded.");

    let display_handle = window.raw_display_handle();

    let required_instance_extensions_raw = match unsafe { ash_window::enumerate_required_extensions(display_handle) } {
        Ok(extensions) => extensions.to_vec(),
        Err(e) => {
            eprintln!("Failed to enumerate required instance extensions: {}", e);
            return;
        }
    };
    let required_instance_extensions_ptr: Vec<*const c_char> = required_instance_extensions_raw
        .iter()
        .map(|&ptr| ptr)
        .collect();
    println!("Required instance extensions obtained.");

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(vk::make_api_version(0, 0, 1, 0))
        .engine_name(&engine_name)
        .engine_version(vk::make_api_version(0, 0, 1, 0))
        .api_version(vk::API_VERSION_1_0);

    let instance_create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_extension_names(&required_instance_extensions_ptr);

    let instance: Instance = match unsafe { entry.create_instance(&instance_create_info, None) } {
        Ok(instance) => instance,
        Err(err) => {
            eprintln!("Failed to create Vulkan instance: {}", err);
            return;
        }
    };
    println!("Vulkan instance created successfully.");

    let surface_loader = AshSurface::new(&entry, &instance);
    let surface = match unsafe {
        ash_window::create_surface(&entry, &instance, display_handle, window.raw_window_handle(), None)
    } {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to create Vulkan surface: {}", e);
            unsafe { instance.destroy_instance(None); }
            return;
        }
    };
    println!("Vulkan surface created.");

    let physical_devices = match unsafe { instance.enumerate_physical_devices() } {
        Ok(devices) => devices,
        Err(err) => {
            eprintln!("Failed to enumerate physical devices: {}", err);
            unsafe {
                surface_loader.destroy_surface(surface, None);
                instance.destroy_instance(None);
            }
            return;
        }
    };
    println!("Found {} physical device(s).", physical_devices.len());

    let (physical_device, queue_family_index) = physical_devices
        .into_iter()
        .find_map(|p_device| {
            let _properties = unsafe { instance.get_physical_device_properties(p_device) };
            let queue_family_properties = unsafe { instance.get_physical_device_queue_family_properties(p_device) };
            queue_family_properties
                .iter()
                .enumerate()
                .find_map(|(index, queue_prop)| {
                    let supports_graphics = queue_prop.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                    let supports_surface = unsafe {
                        surface_loader.get_physical_device_surface_support(p_device, index as u32, surface)
                    }.unwrap_or(false);

                    if supports_graphics && supports_surface {
                        Some((p_device, index as u32))
                    } else {
                        None
                    }
                })
        })
        .expect("No suitable physical device or queue family found.");

    let physical_device_properties = unsafe { instance.get_physical_device_properties(physical_device) };
    println!(
        "Selected physical device: {} (Queue Family Index: {})",
        unsafe { CStr::from_ptr(physical_device_properties.device_name.as_ptr()).to_string_lossy() },
        queue_family_index
    );

    let device_extensions_raw = [
        AshSwapchain::name().as_ptr(),
    ];

    let queue_priorities = [1.0];
    let queue_create_info = vk::DeviceQueueCreateInfo::builder()
        .queue_family_index(queue_family_index)
        .queue_priorities(&queue_priorities);

    let physical_device_features = vk::PhysicalDeviceFeatures::builder();

    let device_create_info = vk::DeviceCreateInfo::builder()
        .queue_create_infos(std::slice::from_ref(&queue_create_info))
        .enabled_extension_names(&device_extensions_raw)
        .enabled_features(&physical_device_features);

    let device: Device = match unsafe { instance.create_device(physical_device, &device_create_info, None) } {
        Ok(dev) => dev,
        Err(err) => {
            eprintln!("Failed to create logical device: {}", err);
            unsafe {
                surface_loader.destroy_surface(surface, None);
                instance.destroy_instance(None);
            }
            return;
        }
    };
    println!("Logical device created successfully.");

    let _graphics_queue = unsafe { device.get_device_queue(queue_family_index, 0) };
    println!("Graphics/Presentation queue handle obtained.");

    let surface_capabilities = unsafe {
        surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
    }.expect("Failed to query surface capabilities.");

    let surface_formats = unsafe {
        surface_loader.get_physical_device_surface_formats(physical_device, surface)
    }.expect("Failed to query surface formats.");

    let present_modes = unsafe {
        surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
    }.expect("Failed to query present modes.");

    let desired_format = surface_formats
        .iter()
        .find(|format| {
            format.format == vk::Format::B8G8R8A8_SRGB && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR
        })
        .unwrap_or_else(|| {
            println!("Desired format B8G8R8A8_SRGB / SRGB_NONLINEAR not found, using first available.");
            &surface_formats[0]
        });

    let desired_present_mode = present_modes
        .iter()
        .find(|&&mode| mode == vk::PresentModeKHR::MAILBOX)
        .unwrap_or(&vk::PresentModeKHR::FIFO);

    let mut desired_image_count = surface_capabilities.min_image_count + 1;
    if surface_capabilities.max_image_count > 0 && desired_image_count > surface_capabilities.max_image_count {
        desired_image_count = surface_capabilities.max_image_count;
    }

    let extent = if surface_capabilities.current_extent.width != u32::MAX {
        surface_capabilities.current_extent
    } else {
        let window_size = window.inner_size();
        vk::Extent2D {
            width: window_size.width.clamp(surface_capabilities.min_image_extent.width, surface_capabilities.max_image_extent.width),
            height: window_size.height.clamp(surface_capabilities.min_image_extent.height, surface_capabilities.max_image_extent.height),
        }
    };

    let swapchain_loader = AshSwapchain::new(&instance, &device);

    let pre_transform = if surface_capabilities.supported_transforms.contains(vk::SurfaceTransformFlagsKHR::IDENTITY) {
        vk::SurfaceTransformFlagsKHR::IDENTITY
    } else {
        surface_capabilities.current_transform
    };

    let swapchain_create_info = vk::SwapchainCreateInfoKHR::builder()
        .surface(surface)
        .min_image_count(desired_image_count)
        .image_format(desired_format.format)
        .image_color_space(desired_format.color_space)
        .image_extent(extent)
        .image_array_layers(1)
        .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
        .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
        .pre_transform(pre_transform)
        .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
        .present_mode(*desired_present_mode)
        .clipped(true);

    let swapchain = match unsafe { swapchain_loader.create_swapchain(&swapchain_create_info, None) } {
        Ok(sc) => sc,
        Err(e) => {
            eprintln!("Failed to create swapchain: {}", e);
            unsafe {
                device.destroy_device(None);
                surface_loader.destroy_surface(surface, None);
                instance.destroy_instance(None);
            }
            return;
        }
    };

    let swapchain_images = match unsafe { swapchain_loader.get_swapchain_images(swapchain) } {
        Ok(images) => images,
        Err(e) => {
            eprintln!("Failed to get swapchain images: {}", e);
            unsafe {
                swapchain_loader.destroy_swapchain(swapchain, None);
                device.destroy_device(None);
                surface_loader.destroy_surface(surface, None);
                instance.destroy_instance(None);
            }
            return;
        }
    };

    let swapchain_image_format = desired_format.format;
    let swapchain_extent = extent;

    let swapchain_image_views: Vec<vk::ImageView> = swapchain_images
        .iter()
        .map(|&image| {
            let image_view_create_info = vk::ImageViewCreateInfo::builder()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(swapchain_image_format)
                .components(vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                })
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });

            unsafe {
                device.create_image_view(&image_view_create_info, None)
            }.expect("Failed to create Image View.")
        })
        .collect();
    println!("Created {} swapchain image views.", swapchain_image_views.len());

    // --- Render Pass Creation ---
    let color_attachment = vk::AttachmentDescription::builder()
        .format(swapchain_image_format)
        .samples(vk::SampleCountFlags::TYPE_1)
        .load_op(vk::AttachmentLoadOp::CLEAR)
        .store_op(vk::AttachmentStoreOp::STORE)
        .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
        .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);

    let color_attachment_ref = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

    let subpass = vk::SubpassDescription::builder()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_ref));

    let dependency = vk::SubpassDependency::builder()
        .src_subpass(vk::SUBPASS_EXTERNAL)
        .dst_subpass(0)
        .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .src_access_mask(vk::AccessFlags::empty()) // Or vk::AccessFlags::MEMORY_READ based on what the previous operation was, if any specific. Empty is fine for UNDEFINED -> WRITE.
        .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
        .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);

    let render_pass_info = vk::RenderPassCreateInfo::builder()
        .attachments(std::slice::from_ref(&color_attachment))
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(std::slice::from_ref(&dependency));

    let render_pass = match unsafe { device.create_render_pass(&render_pass_info, None) } {
        Ok(rp) => rp,
        Err(e) => {
            eprintln!("Failed to create render pass: {}", e);
            unsafe {
                for image_view in &swapchain_image_views { device.destroy_image_view(*image_view, None); }
                swapchain_loader.destroy_swapchain(swapchain, None);
                device.destroy_device(None);
                surface_loader.destroy_surface(surface, None);
                instance.destroy_instance(None);
            }
            return;
        }
    };
    println!("Render pass created successfully.");
    // --- End of Render Pass Creation ---

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                println!("Window close requested. Signaling exit.");
                *control_flow = ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
            }
            Event::LoopDestroyed => {
                println!("Event loop being destroyed.");
            }
            _ => (),
        }

        if *control_flow == ControlFlow::Exit {
            println!("Exiting event loop. Performing final cleanup...");
            unsafe {
                device.device_wait_idle().expect("Failed to wait for device idle.");
                println!("Device idle waited.");

                device.destroy_render_pass(render_pass, None);
                println!("Render pass destroyed.");

                for image_view in &swapchain_image_views {
                    device.destroy_image_view(*image_view, None);
                }
                println!("Swapchain image views destroyed.");

                swapchain_loader.destroy_swapchain(swapchain, None);
                println!("Swapchain destroyed.");

                device.destroy_device(None);
                println!("Logical device destroyed.");

                surface_loader.destroy_surface(surface, None);
                println!("Vulkan surface destroyed.");

                instance.destroy_instance(None);
                println!("Vulkan instance destroyed.");
            }
            println!("Cleanup finished. Application will now exit.");
        }
    });
}
