#include "core/imgui/ImGuiLayer.h"

#include <GLFW/glfw3.h>
#include <glad/glad.h>

#include "core/Application.h"
#include "core/events/Event.h"
#include "platform/opengl/ImGuiOpenGLRenderer.h"

namespace engine {
namespace imgui {

ImGuiLayer::ImGuiLayer() : Layer("ImGuiLayer") {}

ImGuiLayer::~ImGuiLayer() {}

void ImGuiLayer::OnAttach() {
  ImGui::CreateContext();
  ImGui::StyleColorsDark();

  ImGuiIO& io = ImGui::GetIO();
  io.BackendFlags |= ImGuiBackendFlags_HasMouseCursors;
  io.BackendFlags |= ImGuiBackendFlags_HasSetMousePos;

  // TODO(C3NZ): remove this once engine keycodes have been created.
  io.KeyMap[ImGuiKey_Tab] = GLFW_KEY_TAB;
  io.KeyMap[ImGuiKey_LeftArrow] = GLFW_KEY_LEFT;
  io.KeyMap[ImGuiKey_RightArrow] = GLFW_KEY_RIGHT;
  io.KeyMap[ImGuiKey_UpArrow] = GLFW_KEY_UP;
  io.KeyMap[ImGuiKey_DownArrow] = GLFW_KEY_DOWN;
  io.KeyMap[ImGuiKey_PageUp] = GLFW_KEY_PAGE_UP;
  io.KeyMap[ImGuiKey_PageDown] = GLFW_KEY_PAGE_DOWN;
  io.KeyMap[ImGuiKey_Home] = GLFW_KEY_HOME;
  io.KeyMap[ImGuiKey_End] = GLFW_KEY_END;
  io.KeyMap[ImGuiKey_Insert] = GLFW_KEY_INSERT;
  io.KeyMap[ImGuiKey_Delete] = GLFW_KEY_DELETE;
  io.KeyMap[ImGuiKey_Backspace] = GLFW_KEY_BACKSPACE;
  io.KeyMap[ImGuiKey_Space] = GLFW_KEY_SPACE;
  io.KeyMap[ImGuiKey_Enter] = GLFW_KEY_ENTER;
  io.KeyMap[ImGuiKey_Escape] = GLFW_KEY_ESCAPE;
  io.KeyMap[ImGuiKey_KeyPadEnter] = GLFW_KEY_KP_ENTER;
  io.KeyMap[ImGuiKey_A] = GLFW_KEY_A;
  io.KeyMap[ImGuiKey_C] = GLFW_KEY_C;
  io.KeyMap[ImGuiKey_V] = GLFW_KEY_V;
  io.KeyMap[ImGuiKey_X] = GLFW_KEY_X;
  io.KeyMap[ImGuiKey_Y] = GLFW_KEY_Y;
  io.KeyMap[ImGuiKey_Z] = GLFW_KEY_Z;

  ImGui_ImplOpenGL3_Init("#version 410");
}

void ImGuiLayer::OnDetach() {}

// OnUpdate handles
void ImGuiLayer::OnUpdate() {
  ImGuiIO& io = ImGui::GetIO();
  Application& app = Application::GetApplication();
  io.DisplaySize =
      ImVec2(app.GetWindow().GetWidth(), app.GetWindow().GetHeight());

  float time = static_cast<float>(glfwGetTime());
  io.DeltaTime = time_ > 0.0f ? (time - time_) : (1.0f / 60.0f);
  time_ = time;

  ImGui_ImplOpenGL3_NewFrame();
  ImGui::NewFrame();

  static bool show = true;
  ImGui::ShowDemoWindow(&show);

  ImGui::Render();
  ImGui_ImplOpenGL3_RenderDrawData(ImGui::GetDrawData());
}

void ImGuiLayer::OnEvent(events::Event* event) {
  events::EventDispatcher dispatcher(event);

  dispatcher.Dispatch<events::MouseButtonPressedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnMouseButtonPressedEvent));

  dispatcher.Dispatch<events::MouseButtonReleasedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnMouseButtonReleasedEvent));

  dispatcher.Dispatch<events::MouseMovedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnMouseMovedEvent));

  dispatcher.Dispatch<events::MouseScrolledEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnMouseScrolledEvent));

  dispatcher.Dispatch<events::KeyPressedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnKeyPressedEvent));

  dispatcher.Dispatch<events::KeyReleasedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnKeyReleasedEvent));

  dispatcher.Dispatch<events::KeyTypedEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnKeyTypedEvent));

  dispatcher.Dispatch<events::WindowResizeEvent>
      (BIND_EVENT_FN(ImGuiLayer::OnWindowResizeEvent));
}

bool ImGuiLayer::OnMouseButtonPressedEvent(
    const events::MouseButtonPressedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.MouseDown[event.GetMouseButton()] = true;

  return false;
}

bool ImGuiLayer::OnMouseButtonReleasedEvent(
    const events::MouseButtonReleasedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.MouseDown[event.GetMouseButton()] = false;

  return false;
}

bool ImGuiLayer::OnMouseMovedEvent(const events::MouseMovedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.MousePos = ImVec2(event.GetX(), event.GetY());

  return false;
}
bool ImGuiLayer::OnMouseScrolledEvent(const events::MouseScrolledEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.MouseWheel += event.GetYOffset();
  io.MouseWheelH += event.GetXOffset();

  return false;
}

bool ImGuiLayer::OnKeyPressedEvent(const events::KeyPressedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.KeysDown[event.GetKeyCode()] = true;

  io.KeyAlt = io.KeysDown[GLFW_KEY_LEFT_ALT] || io.KeysDown[GLFW_KEY_RIGHT_ALT];
  io.KeyCtrl = io.KeysDown[GLFW_KEY_LEFT_CONTROL]
      || io.KeysDown[GLFW_KEY_RIGHT_CONTROL];
  io.KeyShift = io.KeysDown[GLFW_KEY_LEFT_SHIFT]
      || io.KeysDown[GLFW_KEY_RIGHT_SHIFT];
  io.KeySuper = io.KeysDown[GLFW_KEY_LEFT_SUPER]
      || io.KeysDown[GLFW_KEY_RIGHT_SUPER];

  return false;
}

bool ImGuiLayer::OnKeyReleasedEvent(const events::KeyReleasedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  io.KeysDown[event.GetKeyCode()] = false;

  return false;
}

bool ImGuiLayer::OnKeyTypedEvent(const events::KeyTypedEvent& event) {
  ImGuiIO& io = ImGui::GetIO();
  int key_code = event.GetKeyCode();

  if (key_code > 0 && key_code < 0x10000) {
    io.AddInputCharacter(static_cast<unsigned int16_t>(key_code));
  }

  return false;
}

bool ImGuiLayer::OnWindowResizeEvent(const events::WindowResizeEvent& event) {
  ImGuiIO io = ImGui::GetIO();
  io.DisplaySize = ImVec2(event.GetWidth(), event.GetHeight());
  io.DisplayFramebufferScale = ImVec2(1.0f, 1.0f);
  glViewport(0, 0, event.GetWidth(), event.GetHeight());

  return false;
}


}  // namespace imgui
}  // namespace engine
