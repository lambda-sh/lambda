#include "Lambda/core/Application.h"

#include <functional>

#include <Lambda/core/Window.h>
#include <Lambda/core/events/ApplicationEvent.h>
#include <Lambda/core/events/Event.h>
#include <Lambda/core/input/Input.h>
#include <Lambda/core/layers/Layer.h>
#include <Lambda/core/memory/Pointers.h>
#include <Lambda/core/renderer/Renderer.h>
#include <Lambda/lib/Assert.h>
#include <Lambda/lib/Log.h>
#include <Lambda/lib/Reverse.h>
#include <Lambda/lib/Time.h>
#include <Lambda/profiler/Profiler.h>

namespace lambda::core {

memory::Unique<Application> Application::kApplication_ = nullptr;

/// Will only be instantiated so long as assertions are enabled
/// and there isn't another application instance already running.
Application::Application() {
  LAMBDA_CORE_ASSERT(!kApplication_, "Application already exists.", "");
  LAMBDA_PROFILER_BEGIN_SESSION("Application", "Application.json");
  kApplication_.reset(this);

  window_ = Window::Create();
  window_->SetEventCallback(events::Bind(&Application::OnEvent, this));

  // After the window is setup, initialize the renderer!
  renderer::Renderer::Init();

  /// @todo The ImGUI layer should not be owned by the application.
  imgui_layer_ = memory::CreateUnique<imgui::ImGuiLayer>();
  imgui_layer_->OnAttach();
}

/// The application must tell the single to release itself once it's being
/// destroyed, so that it's destructor is not called again.
Application::~Application() {
  kApplication_.release();
  LAMBDA_PROFILER_END_SESSION();
}

void Application::Run() {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  while (running_) {
    lib::Time current_frame_time;
    lib::TimeStep time_step(last_frame_time_, current_frame_time);
    last_frame_time_ = current_frame_time;

    // Update layers if not minimized
    if (!minimized_) {
      imgui_layer_->OnUpdate(time_step);

      for (auto& layer : layer_stack_) {
        layer->OnUpdate(time_step);
      }
    }

    imgui_layer_->Begin();
    imgui_layer_->OnImGuiRender();
    for (auto& layer : layer_stack_) {
      layer->OnImGuiRender();
    }
    imgui_layer_->End();

    window_->OnUpdate();
  }
}

void Application::OnEvent(memory::Unique<events::Event> event) {
  LAMBDA_PROFILER_MEASURE_FUNCTION();

  events::Dispatcher::HandleWhen<events::WindowCloseEvent>(
      events::Bind(&Application::OnWindowClosed, this), event.get());

  events::Dispatcher::HandleWhen<events::WindowResizeEvent>(
      events::Bind(&Application::OnWindowResize, this), event.get());

  for (auto& layer : lib::Reverse(layer_stack_)) {
    layer->OnEvent(event.get());
    if (event->HasBeenHandled()) {
      break;
    }
  }
}

void Application::PushLayer(memory::Unique<layers::Layer> layer) {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  layer->OnAttach();
  layer_stack_.PushLayer(std::move(layer));
}

void Application::PushOverlay(memory::Unique<layers::Layer> layer) {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  layer->OnAttach();
  layer_stack_.PushOverlay(std::move(layer));
}

bool Application::OnWindowClosed(const events::WindowCloseEvent& event) {
  running_ = false;
  return false;
}

/// Doesn't update when the window is resized.
bool Application::OnWindowResize(const events::WindowResizeEvent& event) {
  if (event.GetWidth() == 0 || event.GetHeight() == 0) {
    minimized_ = true;
    return false;
  }

  // Send the resize to the renderer.
  minimized_ = false;
  renderer::Renderer::OnWindowResize(event.GetWidth(), event.GetHeight());

  return false;
}


}  // namespace lambda::core
