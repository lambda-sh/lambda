use winit::event_loop::{
  ControlFlow,
  EventLoop,
  EventLoopProxy,
  EventLoopWindowTarget,
};

pub enum Event {
  Initialized,
  Shutdown,
  Resized { new_width: u32, new_height: u32 },
}

pub struct LambdaEventLoop {
  event_loop: EventLoop<Event>,
}

pub struct EventLoopPublisher {
  winit_proxy: EventLoopProxy<Event>,
}

impl EventLoopPublisher {
  /// Instantiate a new EventLoopPublisher from an event loop proxy.
  pub fn new(winit_proxy: EventLoopProxy<Event>) -> Self {
    return EventLoopPublisher { winit_proxy };
  }

  pub fn send_event(&self, event: Event) {
    self.winit_proxy.send_event(event);
  }
}

/// A wrapper over the Winit event loop that allows for reuse of their event
/// loops.
impl LambdaEventLoop {
  /// Creates a new Lambda event loop with the underlying event loop
  /// implementation allocated on the heap.
  pub fn new() -> Self {
    let event_loop = EventLoop::<Event>::with_user_event();
    return LambdaEventLoop { event_loop };
  }

  /// Executes the event loop in the current thread.
  pub fn run_forever<Callback>(self, callback: Callback)
  where
    Callback: 'static
      + FnMut(
        winit::event::Event<Event>,
        &EventLoopWindowTarget<Event>,
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
  pub fn winit_loop_ref(&self) -> &EventLoop<Event> {
    return &self.event_loop;
  }
}
