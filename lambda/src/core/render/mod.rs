pub mod pipeline;
pub mod render_pass;
pub mod shader;
pub mod window;

use lambda_platform::{
  gfx::{
    command::{
      CommandPool,
      CommandPoolBuilder,
    },
    fence::{
      RenderSemaphore,
      RenderSemaphoreBuilder,
      RenderSubmissionFence,
      RenderSubmissionFenceBuilder,
    },
    gpu::{
      Gpu,
      RenderQueueType,
    },
    pipeline::RenderPipelineBuilder,
    render_pass::{
      RenderPass,
      RenderPassBuilder,
    },
    surface::{
      Surface,
      SurfaceBuilder,
    },
    GpuBuilder,
    Instance,
    InstanceBuilder,
  },
  shaderc::ShaderKind,
};

pub mod internal {
  use lambda_platform::gfx::api::RenderingAPI as RenderContext;
  pub type RenderBackend = RenderContext::Backend;
}

use shader::Shader;

pub struct RenderAPIBuilder {
  shaders_to_load: Vec<Shader>,
  name: String,
}

impl RenderAPIBuilder {
  pub fn new() -> Self {
    return Self {
      shaders_to_load: vec![],
      name: "lambda".to_string(),
    };
  }

  pub fn with_name(mut self, name: &str) -> Self {
    self.name = name.to_string();
    return self;
  }

  /// Attaches a shader to the renderer to load before being built. This allows
  /// for shaders to be loaded prior to the renderer being initialized.
  pub fn with_shader(mut self, shader: Shader) -> Self {
    self.shaders_to_load.push(shader);
    return self;
  }

  /// Builds a RenderAPI that can be used to access the GPU. Currently only
  /// supports building Graphical Rendering APIs.
  pub fn build(self, window: &window::Window) -> RenderAPI {
    let name = self.name;
    let mut instance =
      InstanceBuilder::new().build::<internal::RenderBackend>(name.as_str());
    let mut surface =
      SurfaceBuilder::new().build(&instance, window.window_handle());

    // Build a GPU with a 3D Render queue that can render to our surface.
    let mut gpu = GpuBuilder::new()
      .with_render_queue_type(RenderQueueType::Graphical)
      .build(&mut instance, Some(&surface))
      .expect("Failed to build a GPU.");

    // Build command pool and allocate a single buffer named Primary
    let mut command_pool = CommandPoolBuilder::new().build(&gpu);
    command_pool.allocate_command_buffer("Primary");

    // Build our rendering submission fence and semaphore.
    let submission_fence = RenderSubmissionFenceBuilder::new()
      .with_render_timeout(1_000_000_000)
      .build(&mut gpu);

    let render_semaphore = RenderSemaphoreBuilder::new().build(&mut gpu);

    let mut render_pass = RenderPassBuilder::new().build(&gpu);

    // Create the image extent and initial frame buffer attachment description
    // for rendering.
    let dimensions = window.dimensions();
    let swapchain_config =
      surface.generate_swapchain_config(&gpu, [dimensions[0], dimensions[1]]);

    let (extent, _frame_buffer_attachment) =
      surface.apply_swapchain_config(&gpu, swapchain_config);

    return RenderAPI {
      name,
      instance,
      gpu,
      surface,
      submission_fence,
      render_semaphore,
      command_pool,
      render_passes: vec![render_pass],
    };
  }
}

/// Generic Rendering API setup to use the current platforms primary
/// Rendering Backend
pub struct RenderAPI {
  name: String,
  instance: Instance<internal::RenderBackend>,
  gpu: Gpu<internal::RenderBackend>,
  surface: Surface<internal::RenderBackend>,
  submission_fence: RenderSubmissionFence<internal::RenderBackend>,
  render_semaphore: RenderSemaphore<internal::RenderBackend>,
  command_pool: CommandPool<internal::RenderBackend>,
  render_passes: Vec<RenderPass<internal::RenderBackend>>,
}

impl RenderAPI {
  pub fn destroy(self) {
    let RenderAPI {
      name,
      submission_fence,
      instance,
      mut gpu,
      mut surface,
      render_semaphore,
      command_pool,
      render_passes,
    } = self;

    println!("{} will now start destroying resources.", name);

    // Destroy the submission fence and rendering semaphore.
    submission_fence.destroy(&gpu);
    render_semaphore.destroy(&gpu);

    command_pool.destroy(&mut gpu);
    for render_pass in render_passes {
      render_pass.destroy(&gpu);
    }

    surface.remove_swapchain_config(&gpu);
    surface.destroy(&instance);
  }

  pub fn render() {}
}

// TODO(vmarcella): Abstract the gfx hal assembler away from the
// render module directly.
// let vertex_entry = gfx_hal_exports::EntryPoint::<B> {
// entry: "main",
// module: &vertex_module,
// specialization: gfx_hal_exports::Specialization::default(),
// };

// let fragment_entry = gfx_hal_exports::EntryPoint::<B> {
// entry: "main",
// module: &fragment_module,
// specialization: gfx_hal_exports::Specialization::default(),
// };

// TODO(vmarcella): This process could use a more consistent abstraction
// for getting a pipeline created.
// let assembler = create_vertex_assembler(vertex_entry);
// let pipeline_layout = self.gpu.create_pipeline_layout();
// let mut logical_pipeline = gfx::pipeline::create_graphics_pipeline(
// assembler,
// &pipeline_layout,
// render_pass,
// Some(fragment_entry),
// );

// let physical_pipeline =
// self.gpu.create_graphics_pipeline(&mut logical_pipeline);

// return (vertex_module, pipeline_layout, physical_pipeline);

// let render_pass = self.gpu.create_render_pass(None, None, None);

// let (module, pipeline_layout, pipeline) =
// self.create_gpu_pipeline(vertex_shader, fragment_shader, &render_pass);
// self.gpu.destroy_shader_module(module);
// self.pipeline_layouts = Some(vec![pipeline_layout]);
// self.graphic_pipelines = Some(vec![pipeline]);

// let surface = self.surface.as_mut().unwrap();

// let acquire_timeout_ns = 1_000_000_000;
//  let image = unsafe {
//    let i = match surface.acquire_image(acquire_timeout_ns) {
//      Ok((image, _)) => Some(image),
//      Err(_) => None,
//    };
//    i.unwrap()
//  };

// TODO(vmarcella): This code will fail if there are no render passes
// attached to the renderer.

// TODO(vmarcella): Investigate into abstracting the viewport behind a
// camera.
// let viewport = {
//  gfx_hal_exports::Viewport {
//    rect: gfx_hal_exports::Rect {
//      x: 0,
//      y: 0,
//      w: self.extent.as_ref().unwrap().width as i16,
//      h: self.extent.as_ref().unwrap().height as i16,
//    },
//    depth: 0.0..1.0,
//  }
// };

// unsafe {
//  let command_buffer = self.command_buffer.as_mut().unwrap();
//  command_buffer
//    .begin_primary(gfx_hal_exports::CommandBufferFlags::ONE_TIME_SUBMIT);
//
// Configure the vieports & the scissor rectangles for the rasterizer
//  let viewports = vec![viewport.clone()].into_iter();
//   let rect = vec![viewport.rect].into_iter();
// command_buffer.set_viewports(0, viewports);
// command_buffer.set_scissors(0, rect);

// Render attachments to specify for the current render pass.
// let render_attachments = vec![gfx_hal_exports::RenderAttachmentInfo {
// image_view: image.borrow(),
// clear_value: gfx_hal_exports::ClearValue {
// color: gfx_hal_exports::ClearColor {
//   float32: [0.0, 0.0, 0.0, 1.0],
// },
//},
//}]
//.into_iter();

// Initialize the render pass on the command buffer & inline the subpass
// contents.
//   command_buffer.begin_render_pass(
//    &render_pass,
//    &framebuffer,
//    viewport.rect,
//    render_attachments,
//    gfx_hal_exports::SubpassContents::Inline,
//  );

// Bind graphical pipeline and submit commands to the GPU.
//   let pipeline = &self.graphic_pipelines.as_ref().unwrap()[0];
//  command_buffer.bind_graphics_pipeline(pipeline);
//  command_buffer.draw(0..3, 0..1);
//  command_buffer.end_render_pass();
//  command_buffer.finish();

// Submit the command buffer for rendering on the GPU.
// self.gpu.submit_command_buffer(
//   &command_buffer,
//   self.rendering_complete_semaphore.as_ref().unwrap(),
//   self.submission_complete_fence.as_mut().unwrap(),
//  );

// let result = self.gpu.render_to_surface(
//   surface,
//   image,
//   self.rendering_complete_semaphore.as_mut().unwrap(),
// );
// if result.is_err() {
//    todo!("Publish an event from the renderer that the swapchain needs to be reconfigured")
// }
