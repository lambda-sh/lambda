use gfx_hal::device::Device;

use super::{gpu::Gpu, surface::ColorFormat};

#[derive(Debug)]
pub enum Operations {
  DontCare,
  Load,
  Clear,
  Store,
}

impl Operations {
  fn to_gfx_hal_load_operation(&self) -> gfx_hal::pass::AttachmentLoadOp {
    match self {
      Operations::DontCare => gfx_hal::pass::AttachmentLoadOp::DontCare,
      Operations::Load => gfx_hal::pass::AttachmentLoadOp::Load,
      Operations::Clear => gfx_hal::pass::AttachmentLoadOp::Clear,
      _ => panic!("Cannot pass in {:?} as an operation for the attachment load operation!", self)
    }
  }

  fn to_gfx_hal_store_operation(&self) -> gfx_hal::pass::AttachmentStoreOp {
    return match self {
      Operations::DontCare => gfx_hal::pass::AttachmentStoreOp::DontCare,
      Operations::Store => gfx_hal::pass::AttachmentStoreOp::Store,
      _ => panic!(
        "Cannot pass in {:?} as an operation for the attachment store operation!",
        self
      ),
    };
  }
}

pub struct AttachmentBuilder {
  samples: u8,
  color_format: Option<ColorFormat>,
  load_operation: gfx_hal::pass::AttachmentLoadOp,
  store_operation: gfx_hal::pass::AttachmentStoreOp,
}

impl AttachmentBuilder {
  pub fn new() -> Self {
    return Self {
      samples: 0,
      color_format: None,
      load_operation: gfx_hal::pass::AttachmentLoadOp::DontCare,
      store_operation: gfx_hal::pass::AttachmentStoreOp::DontCare,
    };
  }

  pub fn with_samples(mut self, samples: u8) -> Self {
    self.samples = samples;
    return self;
  }

  pub fn with_color_format(mut self, color_format: ColorFormat) -> Self {
    self.color_format = Some(color_format);
    return self;
  }

  pub fn on_load(mut self, operation: Operations) -> Self {
    self.load_operation = operation.to_gfx_hal_load_operation();
    return self;
  }

  pub fn on_store(mut self, operation: Operations) -> Self {
    self.store_operation = operation.to_gfx_hal_store_operation();
    return self;
  }

  pub fn build(self) -> Attachment {
    return Attachment {
      attachment: gfx_hal::pass::Attachment {
        format: self.color_format,
        samples: self.samples,
        ops: gfx_hal::pass::AttachmentOps::new(
          self.load_operation,
          self.store_operation,
        ),
        stencil_ops: gfx_hal::pass::AttachmentOps::DONT_CARE,
        layouts: gfx_hal::image::Layout::Undefined
          ..gfx_hal::image::Layout::Present,
      },
    };
  }
}

pub struct Attachment {
  attachment: gfx_hal::pass::Attachment,
}

impl Attachment {
  fn gfx_hal_attachment(&self) -> gfx_hal::pass::Attachment {
    return self.attachment.clone();
  }
}

pub struct RenderPassBuilder {
  attachments: Vec<Attachment>,
}

impl RenderPassBuilder {
  pub fn new() -> Self {
    return Self {
      attachments: vec![],
    };
  }

  /// Adds an attachment to the render pass. Can add multiple.
  pub fn add_attachment(mut self, attachment: Attachment) -> Self {
    self.attachments.push(attachment);
    return self;
  }

  // TODO(vmarcella): implement subpass building logic logic.
  pub fn with_subpass(mut self) -> Self {
    return self;
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    mut self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> RenderPass<RenderBackend> {
    // Build all attachments.
    if self.attachments.is_empty() {
      self.attachments.push(
        AttachmentBuilder::new()
          .with_samples(1)
          .on_load(Operations::Clear)
          .on_store(Operations::Store)
          .build(),
      )
    }

    let attachments = self
      .attachments
      .into_iter()
      .map(|attachment| attachment.gfx_hal_attachment());

    let render_pass = unsafe {
      gpu
        .get_logical_device()
        .create_render_pass(
          attachments.into_iter(),
          vec![].into_iter(),
          vec![].into_iter(),
        )
        .ok()
        .unwrap()
    };

    return RenderPass { render_pass };
  }
}

pub struct RenderPass<RenderBackend: gfx_hal::Backend> {
  render_pass: RenderBackend::RenderPass,
}
