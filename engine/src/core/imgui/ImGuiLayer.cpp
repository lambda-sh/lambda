#include "core/imgui/ImGuiLayer.h"

#include "core/events/Event.h"

namespace engine {
namespace imgui {

ImGuiLayer::ImGuiLayer() : Layer("ImGuiLayer") {}

ImGuiLayer::~ImGuiLayer() {}

void ImGuiLayer::OnAttach() {}
void ImGuiLayer::OnDetach() {}
void ImGuiLayer::OnUpdate() {}

void ImGuiLayer::OnEvent(events::Event* event) {}

}  // namespace imgui
}  // namespace engine
