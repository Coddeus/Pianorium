#version 330 core

in VS_OUTPUT {
    float Color;
    vec2 Position;
} IN;
out vec4 Color;

uniform float u_time;
uniform vec2 u_resolution;

mat2 rotate(float angle)
{
    float c = cos(angle);
    float s = sin(angle);

    return mat2(c, -s, s, c);
}

void main()
{
    if (IN.Color>0.5) {
        Color = vec4(1.0);
    } 
    else {  // Shader Art by lukasxl on ShaderToy
        vec2 uv = IN.Position.xy*vec2(u_resolution.x/u_resolution.y, 1.0); 

        vec4 result = vec4(0,0,0,1);

        float t = 1.;
        float offset = -5. * u_time;
        float base = 100. * length(uv);

        float d = sin(-u_time + 15. * length(uv));
        d *= d * d;

        mat2 rot = rotate(5. * length(uv));
        uv += .5;

        uv = abs(rot * uv);

        for (int p = 0; p < 3; p++)
        {
            result[p] = sin(offset + t * base) - cos(20. * uv.x) - cos(20. * uv.y);
            t += 0.05;
        }

        result.xyz *= result.xyz;
        result.xyz = 1. - result.xyz;

        Color = result * d;
    }
}