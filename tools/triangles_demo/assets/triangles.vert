#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(push_constant) uniform PushConstant {
  vec4 color;
  vec2 pos;
  vec2 scale;
} pcs;

layout(location = 0) out vec4 vertex_color;

vec2 positions[3] = vec2[](
  vec2(0.0, -0.5),
  vec2(-0.5, 0.5),
  vec2(0.5, 0.5)
);

void main() {
  vec2 position = positions[gl_VertexIndex] * pcs.scale;
  vertex_color = pcs.color;
  gl_Position = vec4((position + pcs.pos), 0.0, 1.0);
}
