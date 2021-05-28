#include "Sandbox2D.h"

#include <Lambda/Lambda.h>
#include <glm/glm.hpp>
#include <glm/gtc/type_ptr.hpp>
#include <imgui.h>

/// @todo (C3NZ): Rewrite the shader implementation to not have to rely on the
/// use of platform specific graphic APIs.
#include "Lambda/platform/opengl/OpenGLShader.h"

namespace tools {
namespace sandbox {

namespace {

namespace renderer = lambda::core::renderer;

}  // namespace

/// Calls the parent constructor to give it a debug name.
/// Logic is deliberately kept to the on attach for now to only allocate
/// resources when the layer has been attached to the Application instance.
Sandbox2D::Sandbox2D() :
  Layer("Sandbox2D"),
  camera_controller_(1280.0f / 720.0f) {
  }

void Sandbox2D::OnAttach() {
  checkerboard_texture_ = renderer::Texture2D::Create(
      "assets/textures/checkboard.png");
}

void Sandbox2D::OnDetach() {}

void Sandbox2D::OnUpdate(lambda::lib::TimeStep delta) {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  camera_controller_.OnUpdate(delta);

  renderer::RenderCommand::SetClearColor({ 0.1f, 0.1f, 0.1f, 1.0f });
  renderer::RenderCommand::Clear();

  renderer::Renderer2D::BeginScene(camera_controller_.GetOrthographicCamera());

  renderer::Renderer2D::DrawQuad(
      {-1.0f, 0.0f}, {0.8f, 0.8f}, {0.8f, 0.2f, 0.3f, 1.0f});

  renderer::Renderer2D::DrawQuad(
      {0.5f, -0.5f}, {0.5f, 0.75f}, { 0.2f, 0.3f, 0.8f, 1.0f});

  renderer::Renderer2D::DrawQuad(
      {0.0f, 0.0f}, {10.0f, 10.0f}, checkerboard_texture_);


  renderer::Renderer2D::DrawQuad(
      {10.0f, 10.0f}, quad_size_, checkerboard_texture_);

  // Clamp size so that 0.0f <= x >= 10
  if (quad_size_.x >= 10.0f) {
    quad_size_increasing_ = false;
  } else if (quad_size_.x <= 0.0f) {
    quad_size_increasing_ = true;
  }

  float scaling_factor = 0.009;

  if (quad_size_increasing_) {
    quad_size_ += scaling_factor;
  } else {
    quad_size_ -= scaling_factor;
  }

  renderer::Renderer2D::EndScene();
}

void Sandbox2D::OnImGuiRender() {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  ImGui::Begin("Settings");
  ImGui::ColorEdit3("Shader color", glm::value_ptr(shader_color_));
  ImGui::End();
}

void Sandbox2D::OnEvent(lambda::core::events::Event* const event) {
  LAMBDA_PROFILER_MEASURE_FUNCTION();
  camera_controller_.OnEvent(event);
}

}  // namespace sandbox
}  // namespace tools
