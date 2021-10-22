use winit::{
  event::Event,
  event_loop::{
    ControlFlow,
    EventLoop,
    EventLoopProxy,
    EventLoopWindowTarget,
  },
};

pub enum LambdaEvent {
  Initialized,
  Shutdown,
  Resized { new_width: u32, new_height: u32 },
}

pub struct LambdaEventLoop {
  event_loop: EventLoop<LambdaEvent>,
}

pub struct EventLoopPublisher {
  winit_proxy: EventLoopProxy<LambdaEvent>,
}

impl EventLoopPublisher {
  /// Instantiate a new EventLoopPublisher from an event loop proxy.
  pub fn new(winit_proxy: EventLoopProxy<LambdaEvent>) -> Self {
    return EventLoopPublisher { winit_proxy };
  }

  pub fn send_event(&self, event: LambdaEvent) {
    self.winit_proxy.send_event(event);
  }
}

/// A wrapper over the Winit event loop that allows for reuse of their event
/// loops.
impl LambdaEventLoop {
  /// Creates a new Lambda event loop with the underlying event loop
  /// implementation allocated on the heap.
  pub fn new() -> Self {
    let event_loop = EventLoop::<LambdaEvent>::with_user_event();
    return LambdaEventLoop { event_loop };
  }

  /// Executes the event loop in the current thread.
  pub fn run_forever<Callback>(self, callback: Callback)
  where
    Callback: 'static
      + FnMut(
        Event<LambdaEvent>,
        &EventLoopWindowTarget<LambdaEvent>,
        &mut ControlFlow,
      ) -> (),
  {
    self.event_loop.run(callback);
  }

  /// Creates an event publisher that can be used for publishing events to the
  /// event loop.
  pub fn create_publisher(&self) -> EventLoopPublisher {
    return EventLoopPublisher::new(self.event_loop.create_proxy());
  }

  /// Returns a reference to the underlying winit pointer.
  pub fn winit_loop_ref(&self) -> &EventLoop<LambdaEvent> {
    return &self.event_loop;
  }
}
