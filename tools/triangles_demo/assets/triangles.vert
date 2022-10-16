#version 450
#extension GL_ARB_separate_shader_objects : enable


layout(location = 0) out vec4 vertex_color;

layout( push_constant ) uniform Block {
    vec4 color;
    vec2 position;
    vec2 scale;
} PushConstants;

vec2 positions[3] = vec2[](
  vec2(0.0, -0.5),
  vec2(-0.5, 0.5),
  vec2(0.5, 0.5)
);

void main() {
  vec2 position = positions[gl_VertexIndex] * PushConstants.scale;
  vertex_color = PushConstants.color;
  gl_Position = vec4((position + PushConstants.position), 0.0, 1.0);
}
