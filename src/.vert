#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in float Color;

out VS_OUTPUT {
    float Color;
    vec2 Position;
} OUT;

void main()
{
    gl_Position = vec4(Position, 0.0, 1.0);
    OUT.Color = Color;
    OUT.Position = Position;
}