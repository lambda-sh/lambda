#version 450
#extension GL_ARB_separate_shader_objects : enable

vec2 positions[3];

void main() {
  positions[0] = vec2(0.0, -0.5);
  positions[1] = vec2(-0.5, 0.5);
  positions[2] = vec2(0.5, 0.5);


  vec2 pos = positions[gl_VertexIndex];
  gl_Position = vec4(pos, 0.0, 1.0);
}
