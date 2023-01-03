pub const MATH: &str = r#"

// float3 operations
float3 float3_scaled_by(float3 f3, float s){
    return (float3)(f3.x*s, f3.y*s, f3.z*s);
}

// general numeric operations
float min3(float a, float b, float c){
    return min(a, min(b, c));
}

float max3(float a, float b, float c){
    return max(a, max(b, c));
}

"#;