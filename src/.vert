#version 330 core

layout (location = 0) in vec2 Position;
layout (location = 1) in float Color;

uniform int u_vflip;

out VS_OUTPUT {
    float Color;
    vec2 Position;
} OUT;

void main()
{
    gl_Position = vec4(Position.x, Position.y, 0.0, 1.0);
    if (u_vflip==1) { gl_Position.y *= -1; }
    OUT.Color = Color;
    OUT.Position = Position;
}