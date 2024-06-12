#version 450

layout(location = 0) in vec3 v_position;
layout(location = 1) in vec3 v_color;

layout(push_constant) uniform MVP {
    mat4 proj;
    mat4 view;
    mat4 model;
};

layout(location = 0) out vec3 color;

void main() {
    color = v_color;
    gl_Position = proj * view * model * vec4(v_position, 1.0);
}
