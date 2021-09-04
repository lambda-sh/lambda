
use winit::event_loop::EventLoop;
use winit::event::Event;
use winit::event_loop::ControlFlow;
use winit::event_loop::EventLoopWindowTarget;
use winit::event_loop::EventLoopProxy;

pub enum LambdaEvent{
    Initialized,
    Shutdown,
}

pub struct LambdaEventLoop {
    event_loop: EventLoop<LambdaEvent>
}


pub struct EventLoopPublisher {
    winit_proxy: EventLoopProxy<LambdaEvent>
}

impl EventLoopPublisher {
    pub fn new(winit_proxy: EventLoopProxy<LambdaEvent>) -> Self {
        return EventLoopPublisher{
            winit_proxy,
        };
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
        return LambdaEventLoop{
            event_loop,
        };
    }

    /// Executes the event loop in the current thread.
    pub fn run_forever<Callback>(self, callback: Callback)
            where Callback: 
                    'static +
                    FnMut(
                        Event<LambdaEvent>, 
                        &EventLoopWindowTarget<LambdaEvent>, 
                        &mut ControlFlow) -> () {
        self.event_loop.run(callback);
    }

    pub fn create_publisher(&self) -> EventLoopPublisher {
        return EventLoopPublisher::new(self.event_loop.create_proxy());
    }

    pub fn winit_loop_ref(&self) -> &EventLoop<LambdaEvent> {
        return &self.event_loop;
    }


}
