in vec3 position;
in vec3 color;
in vec3 instance_position;
in vec3 scale;

uniform mat4 projection;
uniform mat4 view;

out vec3 v_color;

void main() {
  gl_Position = projection * view * vec4(position * scale + instance_position, 1.);
  v_color = color;
}
