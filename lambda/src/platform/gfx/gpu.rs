use std::mem::size_of;

use gfx_hal::{adapter::{
    Adapter,
    Gpu,
  }, command::Level, device::Device, image::{
    Access,
    Layout,
  }, memory::Dependencies, pass::{
    Attachment,
    AttachmentLoadOp,
    AttachmentOps,
    AttachmentStoreOp,
    SubpassDependency,
    SubpassDesc,
  }, pool::{
    CommandPool,
    CommandPoolCreateFlags,
  }, prelude::{
    PhysicalDevice,
    QueueFamily,
  }, pso::{DescriptorSetLayoutBinding, EntryPoint, PipelineCreationFlags, PipelineStage, ShaderStageFlags, Specialization}, window::Surface};

use crate::core::render::shader::LambdaShader;

///
/// Commands oriented around creating resources on & for the GPU.
///
pub struct GfxGpu<B: gfx_hal::Backend> {
  adapter: Adapter<B>,
  gpu: Gpu<B>,
  command_pool: Option<B::CommandPool>,
}

#[derive(Clone, Copy)]
pub enum RenderQueueType {
  Compute,
  Graphical,
  GraphicalCompute,
  Transfer,
}



/// Checks if queue_family is capable of supporting the requested queue type &
/// Optional surface.
fn is_queue_family_supported<B: gfx_hal::Backend>(
  queue_family: &B::QueueFamily,
  queue_type: RenderQueueType,
  surface: Option<&B::Surface>,
) -> bool {
  match queue_type {
    RenderQueueType::Compute => queue_family.queue_type().supports_compute(),
    RenderQueueType::Graphical => match surface {
      Some(surface) => {
        surface.supports_queue_family(queue_family)
          && queue_family.queue_type().supports_graphics()
      }
      None => false,
    },
    // TODO(vmarcella): These arms should be filled out to support the other kinds of queue types.
    RenderQueueType::GraphicalCompute => {
      todo!("GraphicalCompute RenderQueue's are not currently implemented.")
    }
    RenderQueueType::Transfer => {
      todo!("Transfer RenderQueues are not currently implemented.")
    }
  }
}

impl<B: gfx_hal::Backend> GfxGpu<B> {
  /// Instantiates a new GPU given an adapter that is implemented by the GPUs
  /// current rendering backend B. A new GPU does not come with a command pool unless specified.
  pub fn new(
    adapter: Adapter<B>,
    queue_type: RenderQueueType,
    surface: Option<&B::Surface>,
  ) -> Self {
    let queue_family = adapter
      .queue_families
      .iter()
      .find(|family| {
        return is_queue_family_supported::<B>(family, queue_type, surface);
      })
      .expect("No compatible queue family found.");

    let gpu = unsafe {
      adapter
        .physical_device
        .open(&[(queue_family, &[1.0])], gfx_hal::Features::empty())
        .expect("Failed to open the device.")
    };

    return Self {
      adapter,
      gpu,
      command_pool: None,
    };
  }

  /// Attaches a command pool to the current gfx gpu.
  pub fn with_command_pool(self) -> Self {
    let adapter = self.adapter;
    let mut gpu = self.gpu;
    let command_pool = unsafe {
      gpu.device.create_command_pool(
				gpu.queue_groups.pop().unwrap().family,
				CommandPoolCreateFlags::empty())
				.expect("The GPU could not allocate a command pool because it is out of memory")
    };

    return Self {
      adapter,
      gpu,
      command_pool: Some(command_pool),
    };
  }

  /// Allocate's a command buffer through the GPU
  pub fn allocate_command_buffer(&mut self) -> B::CommandBuffer {
    // TODO(vmarcella): This function should probably not just panic and instead
    // return a Result Type that allows for this action to be recoverable if
    // failed.
    return match &mut self.command_pool {
      Some(command_pool) => unsafe {
        command_pool.allocate_one(Level::Primary)
      },
      None => panic!(
        "Cannot allocate a command buffer without a command pool initialized."
      ),
    };
  }

  /// Create a pipeline layout on the GPU.
  pub fn create_pipeline_layout(&mut self) -> B::PipelineLayout {
    unsafe {
      // wait, I think I have a hack for this
      let max: u32 = size_of::<u32>() as u32;
      return self
        .gpu
        .device
        .create_pipeline_layout(
          vec![].iter(),
          vec![(ShaderStageFlags::VERTEX, 0..max)].into_iter(),
        )
        .expect("Out of memory.");
    }
  }

  /// Create a render pass with the current using the current GPU resources.
  pub fn create_render_pass(
    &mut self,
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

    // TODO(vmarcella): Error handling here should propagate an Error upwards.
    return unsafe {
      self
				.gpu
				.device
				.create_render_pass(attachments.into_iter(), subpasses.into_iter(), deps.into_iter())
				.expect("Your primary graphics card does not have enough memory for this render pass.")
    };
  }

	pub fn create_shader_module(&mut self, binary: &Vec<u32>) -> (B::ShaderModule, EntryPoint<B>) {
		unsafe {
			let module = self.gpu.device.create_shader_module(&binary).expect("Failed to create a shader module.");
			// TODO(vmarcella): Allow for the customization of main & 
			let entry = EntryPoint{
				entry: "main",
				module: &module,
				specialization: Specialization::default()
			};

			return (&module, entry);
		}
	}

	pub fn create_render_pipeline(&mut self, render_pass: &B::RenderPass, pipeline_layout: &B::PipelineLayout, shader_modules: Vec<B::ShaderModule> ) {

	}

}