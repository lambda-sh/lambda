use std::borrow::Borrow;

use gfx_hal::{
  device::Device,
  pass::SubpassDesc,
};

use super::{
  gpu::Gpu,
  surface::ColorFormat,
};

// ----------------------- RENDER ATTACHMENT OPERATIONS ------------------------

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

// ----------------------------- RENDER ATTACHMENT -----------------------------

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

// ------------------------------ RENDER SUBPASS -------------------------------

pub use gfx_hal::image::Layout as ImageLayoutHint;

pub struct SubpassBuilder {
  color_attachment: Option<(usize, ImageLayoutHint)>,
}

impl SubpassBuilder {
  pub fn new() -> Self {
    return Self {
      color_attachment: None,
    };
  }

  pub fn use_color_attachment(
    mut self,
    attachment_index: usize,
    layout: ImageLayoutHint,
  ) -> Self {
    self.color_attachment = Some((attachment_index, layout));
    return self;
  }
  pub fn with_inputs() {
    todo!("Implement input support for subpasses")
  }
  pub fn with_resolves() {
    todo!("Implement resolving support for subpasses")
  }
  pub fn with_preserves() {
    todo!("Implement preservation support for subpasses")
  }

  pub fn build<'a>(self) -> Subpass<'a> {
    return Subpass {
      subpass: gfx_hal::pass::SubpassDesc {
        colors: &[(0, ImageLayoutHint::ColorAttachmentOptimal)],
        depth_stencil: None,
        inputs: &[],
        resolves: &[],
        preserves: &[],
      },
    };
  }
}

pub struct Subpass<'a> {
  subpass: gfx_hal::pass::SubpassDesc<'a>,
}

impl<'a> Subpass<'a> {
  fn gfx_hal_subpass(self) -> gfx_hal::pass::SubpassDesc<'a> {
    return self.subpass;
  }
}

// -------------------------------- RENDER PASS --------------------------------

pub struct RenderPassBuilder<'builder> {
  attachments: Vec<Attachment>,
  subpasses: Vec<Subpass<'builder>>,
}

impl<'builder> RenderPassBuilder<'builder> {
  pub fn new() -> Self {
    return Self {
      attachments: vec![],
      subpasses: vec![],
    };
  }

  /// Adds an attachment to the render pass. Can add multiple.
  pub fn add_attachment(mut self, attachment: Attachment) -> Self {
    self.attachments.push(attachment);
    return self;
  }

  pub fn add_subpass(mut self, subpass: Subpass<'builder>) -> Self {
    self.subpasses.push(subpass);
    return self;
  }

  pub fn build<RenderBackend: gfx_hal::Backend>(
    self,
    gpu: &mut Gpu<RenderBackend>,
  ) -> RenderPass<RenderBackend> {
    // Build all attachments.
    let attachments = match self.attachments.is_empty() {
      true => vec![AttachmentBuilder::new()
        .with_samples(1)
        .on_load(Operations::Clear)
        .on_store(Operations::Store)
        .with_color_format(ColorFormat::Rgba8Srgb)
        .build()
        .gfx_hal_attachment()],
      false => self
        .attachments
        .into_iter()
        .map(|attachment| attachment.gfx_hal_attachment())
        .collect(),
    };

    let subpasses = match self.subpasses.is_empty() {
      true => vec![SubpassBuilder::new().build().gfx_hal_subpass()],
      false => self
        .subpasses
        .into_iter()
        .map(|subpass| subpass.gfx_hal_subpass())
        .collect(),
    };

    let render_pass = unsafe {
      gpu
        .get_logical_device()
        .create_render_pass(
          attachments.into_iter(),
          subpasses.into_iter(),
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
