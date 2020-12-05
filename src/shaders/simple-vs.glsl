in vec3 position;
in vec3 color;
uniform mat4 projection;
uniform mat4 view;

out vec3 v_color;

void main() {
  gl_Position = projection * view * vec4(position, 1.);
  v_color = color;
}
