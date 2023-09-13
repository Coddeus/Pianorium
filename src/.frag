#version 330 core

in VS_OUTPUT {
    float Color;
    vec2 Position;
} IN;
out vec4 Color;

uniform float u_time;
uniform vec2 u_resolution;
uniform vec3 u_ol_color;
uniform vec3 u_note_left;
uniform vec3 u_note_right;
uniform vec3 u_note_top;
uniform vec3 u_note_bottom;
uniform vec3 u_note_time;
uniform vec3 u_particle_left;
uniform vec3 u_particle_right;
uniform vec3 u_particle_top;
uniform vec3 u_particle_bottom;
uniform vec3 u_particle_time;

mat2 rotate(float angle)
{
    float c = cos(angle);
    float s = sin(angle);

    return mat2(c, -s, s, c);
}

void main()
{
    vec2 uv = vec2(u_resolution.x*IN.Position.x, u_resolution.y*IN.Position.y); 
    if (IN.Color == 1.0) {
        Color = vec4((mix(u_note_left, u_note_right, (IN.Position.x+1.0)/2.0) * mix(u_note_bottom, u_note_top, (IN.Position.y+1.0)/2.0))*0.5, 1.0);
    } 
    else if (IN.Color == 0.9) {  
        Color = vec4((mix(u_note_left, u_note_right, (IN.Position.x+1.0)/2.0) * mix(u_note_bottom, u_note_top, (IN.Position.y+1.0)/2.0)), 1.0);

        /* // Shader Art by lukasxl on ShaderToy
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
        */
    }
    else if (IN.Color == 0.8) {
        Color = vec4((mix(u_particle_left, u_particle_right, (IN.Position.x+1.0)/2.0) * mix(u_particle_bottom, u_particle_top, (IN.Position.y+1.0)/2.0)), 0.3);
    }
    else if (IN.Color == 0.7) {
        Color = vec4(u_ol_color, 1.0);
    }
}