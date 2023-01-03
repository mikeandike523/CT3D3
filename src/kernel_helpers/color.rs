pub const COLOR: &str = r#"

typedef struct Color {
    float r;
    float g;
    float b;
} Color;

Color color_new(float r, float g, float b){
    Color c;
    c.r = r;
    c.g = g;
    c.b = b;
    return c;
}

Color color_from_float3(float3 f3){
    return color_new(f3.x, f3.y, f3.z);
}

"#;