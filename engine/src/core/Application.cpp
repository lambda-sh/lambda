#include "core/Application.h"

#include <chrono>
#include <functional>
#include <initializer_list>
#include <memory>

#include "core/Input.h"
#include "core/Layer.h"
#include "core/Window.h"
#include "core/events/ApplicationEvent.h"
#include "core/events/Event.h"
#include "core/memory/Pointers.h"
#include "core/util/Assert.h"
#include "core/util/Log.h"
#include "core/util/Reverse.h"
#include "core/util/Time.h"

namespace engine {

memory::Unique<Application> Application::kApplication_ = nullptr;

Application::Application() {
  ENGINE_CORE_ASSERT(!kApplication_, "Application already exists.");
  kApplication_.reset(this);

  window_ = Window::Create();
  window_->SetEventCallback(BIND_EVENT_FN(Application::OnEvent));

  imgui_layer_ = memory::CreateShared<imgui::ImGuiLayer>();
  PushLayer(imgui_layer_);
}

Application::~Application() {}

/**
 * This currently does a lot of custom rendering when in reality it should
 * be implemented by a child project that is running the game. This will change
 * in the future, but at the moment implements a lot of specific rendering
 * tests that are for ensuring that the renderer currently works.
 */
void Application::Run() {
  while (running_) {
    util::Time current_frame_time;
    util::TimeStep time_step(last_frame_time_, current_frame_time);

    last_frame_time_ = current_frame_time;
    for (memory::Shared<Layer> layer : layer_stack_) {
      layer->OnUpdate(time_step);
    }

    imgui_layer_->Begin();
    for (memory::Shared<Layer> layer : layer_stack_) {
      layer->OnImGuiRender();
    }
    imgui_layer_->End();

    window_->OnUpdate();
  }
}

/**
 * This function only specifically listens for when the window is requested to
 * close before passing the event to layers on the LayerStack.
 */
void Application::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);
  dispatcher.Dispatch<events::WindowCloseEvent>
      (BIND_EVENT_FN(Application::OnWindowClosed));

  for (memory::Shared<Layer> layer : util::Reverse(layer_stack_)) {
    layer->OnEvent(event);
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

void Application::PushLayer(memory::Shared<Layer> layer) {
  layer_stack_.PushLayer(layer);
  layer->OnAttach();
}

void Application::PushOverlay(memory::Shared<Layer> layer) {
  layer_stack_.PushOverlay(layer);
  layer->OnAttach();
}

bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return true;
}

}  // namespace engine
