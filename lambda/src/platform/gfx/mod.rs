use std::ops::Range;

use gfx_hal::{
  adapter::{
    Adapter,
    Gpu,
  },
  device::Device,
  format::{
    ChannelType,
    Format,
  },
  image::{
    Access,
    Layout,
  },
  memory::Dependencies,
  pass::{
    Attachment,
    AttachmentLoadOp,
    AttachmentOps,
    AttachmentStoreOp,
    SubpassDependency,
    SubpassDesc,
  },
  prelude::QueueFamily,
  pso::PipelineStage,
  window::Surface,
  Instance,
};

use crate::core::window::LambdaWindow;

/// Create a Surface for a backend using a WinitP.
/// TODO(vmarcella): The window pointer taken in should most likely be a
/// generic.
pub fn create_surface<B: gfx_hal::Backend>(
  instance: &B::Instance,
  window: &LambdaWindow,
) -> B::Surface {
  return unsafe {
    // TODO(vmarcella): This currently just unwraps the surface
    instance
      .create_surface(window.winit_window_ref().unwrap())
      .unwrap()
  };
}

pub fn find_supported_render_queue<'a, B: gfx_hal::Backend>(
  surface: &'a B::Surface,
  adapter: &'a Adapter<B>,
) -> &'a B::QueueFamily {
  return adapter
    .queue_families
    .iter()
    .find(|family| {
      let supports_queue = surface.supports_queue_family(family);
      let supports_graphics = family.queue_type().supports_graphics();

      supports_queue && supports_graphics
    })
    .unwrap();
}

/// Finds the first supported color format or default to Rgba8Srgb.
pub fn find_supported_color_format<B: gfx_hal::Backend>(
  surface: &B::Surface,
  adapter: &Adapter<B>,
) -> Format {
  // Define a surface color format compatible with the graphics
  // device & surface
  let supported_formats = surface
    .supported_formats(&adapter.physical_device)
    .unwrap_or(vec![]);

  let default_format = *supported_formats.get(0).unwrap_or(&Format::Rgba8Srgb);

  return supported_formats
    .into_iter()
    .find(|format| -> bool { format.base_format().1 == ChannelType::Srgb })
    .unwrap_or(default_format);
}

// Creates a render pass given a GPU and optional resources, subpasses, and
// dependencies that you'd like to be specified for the pipeline.
pub fn create_render_pass<B: gfx_hal::Backend>(
  gpu: &mut Gpu<B>,
  resource_attachments: Option<Vec<Attachment>>,
  render_subpasses: Option<Vec<SubpassDesc>>,
  dependencies: Option<Vec<SubpassDependency>>,
) -> B::RenderPass {
  // Use attached resources or create a stub for pipeline compatibility
  let attachments = match resource_attachments {
    Some(attachments) => attachments,
    None => {
      vec![Attachment {
        format: None,
        samples: 1,
        ops: AttachmentOps::new(
          AttachmentLoadOp::Clear,
          AttachmentStoreOp::Store,
        ),
        stencil_ops: AttachmentOps::DONT_CARE,
        layouts: Layout::Undefined..Layout::Present,
      }]
    }
  };

  // Use attached render subpasses or create a stub for pipeline compatibility.
  let subpasses = match render_subpasses {
    Some(subpasses) => subpasses,
    None => vec![SubpassDesc {
      colors: &[(0, Layout::ColorAttachmentOptimal)],
      depth_stencil: None,
      inputs: &[],
      resolves: &[],
      preserves: &[],
    }],
  };

  // Use attached dependencies or create a stub for pipeline compatibility.
  let deps = match dependencies {
    Some(deps) => deps,
    None => vec![SubpassDependency {
      accesses: Access::COLOR_ATTACHMENT_READ..Access::COLOR_ATTACHMENT_WRITE,
      flags: Dependencies::empty(),
      passes: None..None,
      stages: PipelineStage::BOTTOM_OF_PIPE
        ..PipelineStage::COLOR_ATTACHMENT_OUTPUT,
    }],
  };

  return unsafe {
    gpu
				.device
				.create_render_pass(attachments.into_iter(), subpasses.into_iter(), deps.into_iter())
				.expect("Your primary graphics card does not have enough memory for this render pass.")
  };
}
