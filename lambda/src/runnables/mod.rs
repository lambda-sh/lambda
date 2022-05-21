use std::{
  collections::HashMap,
  mem::swap,
  time::Instant,
};

use lambda_platform::{
  gfx,
  gfx::{
    gfx_hal_exports,
    gpu::RenderQueueType,
    surface::destroy_surface,
    GpuBuilder,
  },
  winit::{
    create_event_loop,
    winit_exports::{
      ControlFlow,
      Event as WinitEvent,
      WindowEvent,
    },
    Loop,
  },
};

use crate::{
  components::{
    self,
    ComponentStack,
    Window,
  },
  core::{
    component::{
      self,
      Component,
    },
    events::Event,
    render::{
      self,
      RenderAPI,
    },
    runnable::Runnable,
  },
};

pub fn delete_all_resources<B: gfx_hal_exports::Backend>(
  surface: &mut gfx::surface::Surface<B>,
  gpu: &mut gfx::gpu::Gpu<B>,
  submission_fence: B::Fence,
  rendering_semaphore: B::Semaphore,
  graphics_pipelines: Vec<B::GraphicsPipeline>,
  pipeline_layouts: Vec<B::PipelineLayout>,
  render_passes: Vec<B::RenderPass>,
) {
  println!("Destroying GPU resources allocated during run.");

  gpu.destroy_access_fences(submission_fence, rendering_semaphore);

  for pipeline_layout in pipeline_layouts.into_iter() {
    gpu.destroy_pipeline_layout(pipeline_layout);
  }

  for render_pass in render_passes.into_iter() {
    gpu.destroy_render_pass(render_pass);
  }

  for pipeline in graphics_pipelines.into_iter() {
    gpu.destroy_graphics_pipeline(pipeline);
  }

  // Destroy command pool allocated on the GPU.
  gpu.destroy_command_pool();
  surface.remove_swapchain_config(&gpu);

  println!("Destroyed all GPU resources");
}

///
/// LambdaRunnable is a pre configured composition of a generic set of
/// components from the lambda-rs codebase
pub struct LambdaRunnable {
  name: String,
  event_loop: Loop<Event>,
  window: Window,
  component_stack: ComponentStack,
  instance: gfx::GfxInstance<gfx::api::RenderingAPI::Backend>,
}

impl LambdaRunnable {
  /// Set the name for the current runnable
  pub fn with_name(mut self, name: &str) -> Self {
    self.name = String::from(name);
    return self;
  }

  /// Attach a component to the current runnable.
  pub fn with_component<T: Default + Component + 'static>(
    self,
    configure_component: impl FnOnce(Self, T) -> (Self, T),
  ) -> Self {
    let (mut runnable, component) = configure_component(self, T::default());
    runnable.component_stack.push_component(component);
    return runnable;
  }
}

impl Default for LambdaRunnable {
  /// Constructs a LambdaRunanble with an event loop for publishing events to
  /// the application, a window with a renderable surface, a layer stack for
  /// storing layers into the engine.
  fn default() -> Self {
    let name = String::from("LambdaRunnable");
    let mut event_loop = create_event_loop::<Event>();
    let window = Window::new(name.as_str(), [480, 360], &mut event_loop);
    let component_stack = ComponentStack::new();
    let mut instance = lambda_platform::gfx::create_default_gfx_instance();

    return LambdaRunnable {
      name,
      event_loop,
      window,
      component_stack,
      instance,
    };
  }
}

impl Runnable for LambdaRunnable {
  /// One setup to initialize the
  fn setup(&mut self) {}

  /// Initiates an event loop that captures the context of the LambdaRunnable
  /// and generates events from the windows event loop until the end of an
  /// applications lifetime.
  fn run(self) {
    // Decompose Runnable components for transferring ownership to the
    // closure.
    let app = self;
    let mut window = app.window;
    let name = app.name;
    let mut event_loop = app.event_loop;

    // TODO(vmarcella): The renderer should most likely just act as
    let mut component_stack = app.component_stack;
    let mut instance = app.instance;

    let mut surface = Some(instance.create_surface(window.window_handle()));

    let builder = GpuBuilder::new(&mut instance)
      .with_render_queue_type(RenderQueueType::Graphical);

    let mut gpu = builder
      .build(surface.as_ref())
      .expect("Failed to setup a GPU for lambda");

    let (submission_fence, rendering_semaphore) = gpu.create_access_fences();

    let mut s_fence = Some(submission_fence);
    let mut r_fence = Some(rendering_semaphore);

    // Create the image extent and initial frame buffer attachment description for rendering.
    let dimensions = window.dimensions();
    let swapchain_config = surface
      .as_ref()
      .unwrap()
      .generate_swapchain_config(&gpu, [dimensions[0], dimensions[1]]);
    let (extent, _frame_buffer_attachment) = surface
      .as_mut()
      .unwrap()
      .apply_swapchain_config(&gpu, swapchain_config);

    let publisher = event_loop.create_publisher();
    publisher.send_event(Event::Initialized);

    let mut last_frame = Instant::now();
    let mut current_frame = Instant::now();

    event_loop.run_forever(move |event, _, control_flow| match event {
      WinitEvent::WindowEvent { event, .. } => match event {
        WindowEvent::CloseRequested => {
          // Issue a Shutdown event to deallocate resources and clean up.
          publisher.send_event(Event::Shutdown);
        }
        WindowEvent::Resized(dims) => publisher.send_event(Event::Resized {
          new_width: dims.width,
          new_height: dims.height,
        }),
        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => publisher
          .send_event(Event::Resized {
            new_width: new_inner_size.width,
            new_height: new_inner_size.height,
          }),
        WindowEvent::Moved(_) => {}
        WindowEvent::Destroyed => {}
        WindowEvent::DroppedFile(_) => {}
        WindowEvent::HoveredFile(_) => {}
        WindowEvent::HoveredFileCancelled => {}
        WindowEvent::ReceivedCharacter(_) => {}
        WindowEvent::Focused(_) => {}
        WindowEvent::KeyboardInput {
          device_id,
          input,
          is_synthetic,
        } => {}
        WindowEvent::ModifiersChanged(_) => {}
        WindowEvent::CursorMoved {
          device_id,
          position,
          modifiers,
        } => {}
        WindowEvent::CursorEntered { device_id } => {}
        WindowEvent::CursorLeft { device_id } => {}
        WindowEvent::MouseWheel {
          device_id,
          delta,
          phase,
          modifiers,
        } => {}
        WindowEvent::MouseInput {
          device_id,
          state,
          button,
          modifiers,
        } => {}
        WindowEvent::TouchpadPressure {
          device_id,
          pressure,
          stage,
        } => {}
        WindowEvent::AxisMotion {
          device_id,
          axis,
          value,
        } => {}
        WindowEvent::Touch(_) => {}
        WindowEvent::ThemeChanged(_) => {}
      },
      WinitEvent::MainEventsCleared => {
        last_frame = current_frame.clone();
        current_frame = Instant::now();
        let duration = &current_frame.duration_since(last_frame);
        component_stack.on_update(duration);
      }
      WinitEvent::RedrawRequested(_) => {
        window.redraw();
      }
      WinitEvent::NewEvents(_) => {}
      WinitEvent::DeviceEvent { device_id, event } => {}
      WinitEvent::UserEvent(lambda_event) => {
        match lambda_event {
          Event::Initialized => {
            component_stack.on_attach();
          }
          Event::Shutdown => {
            // Once this has been set, the ControlFlow can no longer be
            // modified.
            *control_flow = ControlFlow::Exit;
          }
          _ => {
            component_stack.on_event(&lambda_event);
          }
        }
      }
      WinitEvent::Suspended => {}
      WinitEvent::Resumed => {}
      WinitEvent::RedrawEventsCleared => {}
      WinitEvent::LoopDestroyed => {
        component_stack.on_detach();
        delete_all_resources(
          surface.as_mut().unwrap(),
          &mut gpu,
          s_fence.take().unwrap(),
          r_fence.take().unwrap(),
          vec![],
          vec![],
          vec![],
        );
        destroy_surface(&instance, surface.take().unwrap());
      }
    });
  }
}

/// Create a generic lambda runnable. This provides you a Runnable
/// Application Instance that can be hooked into through attaching
/// a Layer
pub fn create_lambda_runnable() -> LambdaRunnable {
  return LambdaRunnable::default();
}
