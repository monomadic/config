//!HOOK MAIN
//!BIND HOOKED
//!DESC edge-fade

#define EDGE_FADE_WIDTH 0.105
#define EDGE_FADE_POWER 1.35

vec4 hook()
{
    vec4 color = HOOKED_texOff(0);
    vec2 uv = HOOKED_pos;

    float left = smoothstep(0.0, EDGE_FADE_WIDTH, uv.x);
    float right = smoothstep(0.0, EDGE_FADE_WIDTH, 1.0 - uv.x);
    float bottom = smoothstep(0.0, EDGE_FADE_WIDTH, uv.y);
    float top = smoothstep(0.0, EDGE_FADE_WIDTH, 1.0 - uv.y);

    float fade = pow(left * right * bottom * top, EDGE_FADE_POWER);
    color.rgb *= fade;

    return color;
}
