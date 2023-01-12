
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

float3 float3_tbt_mul(float3 a, float3 b){
    return (float3)(a.x*b.x, a.y*b.y, a.z*b.z);
}

float3 towards_east(float amount){
    return float3_scaled_by((float3)(1.0, 0.0, 0.0),amount);
}

float3 towards_north(float amount){
    return float3_scaled_by((float3)(0.0, 0.0, 1.0),amount);
}

float3 towards_west(float amount){
    return float3_scaled_by((float3)(-1.0, 0.0, 0.0),amount);
}

float3 towards_south(float amount){
    return float3_scaled_by((float3)(0.0, 0.0, -1.0),amount);
}

float3 towards_up(float amount){
    return float3_scaled_by((float3)(0.0, 1.0, 0.0),amount);
}

float3 towards_down(float amount){
    return float3_scaled_by((float3)(0.0, -1.0, 0.0),amount);
}

float3 safe_normalize(float3 a){
    float m = length(a);
    if(length(a)<1e-9){
        return (float3)(0.0,0.0,0.0);
    }
    return float3_scaled_by(a, 1.0/m);
}


