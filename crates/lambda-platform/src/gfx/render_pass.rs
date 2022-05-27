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
  samples: u32,
  load_operation: gfx_hal::pass::AttachmentLoadOp,
  store_operation: gfx_hal::pass::AttachmentStoreOp,
}

impl AttachmentBuilder {
  pub fn new() -> Self {
    return Self {
      samples: 0,
      load_operation: gfx_hal::pass::AttachmentLoadOp::DontCare,
      store_operation: gfx_hal::pass::AttachmentStoreOp::DontCare,
    };
  }

  pub fn with_samples(mut self, samples: u32) -> Self {
    self.samples = samples;
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
}

pub struct Attachment {
  attachment: gfx_hal::pass::Attachment,
}

pub struct RenderPassBuilder {}

pub struct RenderPass<RenderBackend: gfx_hal::Backend> {
  render_pass: RenderBackend::RenderPass,
}
