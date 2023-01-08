pub const RAYCASTING: &str = r#"

#define RAYCASTING_DENOM_EPSILON 1.0e-6

#define FLOAT3_EAST ((float3)(1.0,0.0,0.0))
#define FLOAT3_NORTH ((float3)(0.0,0.0,1.0))
#define FLOAT3_WEST ((float3)(-1.0,0.0,0.0))
#define FLOAT3_SOUTH ((float3)(0.0,0.0,-1.0))
#define FLOAT3_UP ((float3)(0.0,1.0,0.0))
#define FLOAT3_DOWN ((float3)(0.0,-1.0,0.0))

typedef struct OptFloat3 {
    int present;
    float3 value;
} OptFloat3;

OptFloat3 OptFloat3_new(int present, float3 value) {
    OptFloat3 opt;
    opt.present = present;
    opt.value = value;
    return opt;
}

OptFloat3 OptFloat3_miss(){
    return OptFloat3_new(0, (float3)(0.0,0.0,0.0));
}

OptFloat3 OptFloat3_hit(float3 value){
    return OptFloat3_new(1, value);
}

OptFloat3 ray_plane_intersection(float3 ro, float3 rd, float3 O, float3 N){

    // N.(ro + t*rd - O) = 0
    // N.trd = N.O - N.ro
    // t = (N.O - N.ro) / (N.rd)


    float denom = dot(N, rd);
    if(fabs(denom) < RAYCASTING_DENOM_EPSILON){
        return OptFloat3_miss();
    }

    float t = (dot(N,O) - dot(N,ro))/denom;

    return OptFloat3_hit(ro+float3_scaled_by(rd,t));
}

OptFloat3 ray_panel_intersection(float3 ro, float3 rd, float3 O, float3 N, float3 T, float3 B, float rT, float rB){
    OptFloat3 of3 = ray_plane_intersection(ro, rd, O, N);
    if(of3.present == 0){
        return OptFloat3_miss();
    }
    float tB = dot(of3.value-O,B);
    float tT = dot(of3.value-O,T);
    if(fabs(tB)>rB || fabs(tT)>rT){
        return OptFloat3_miss();
    }
    return of3;
}

OptFloat3 minimum_distance_union(OptFloat3 of31, OptFloat3 of32, float3 ro){
    if(of31.present == 0 && of32.present == 0){
        return OptFloat3_miss();
    }
    if(of31.present == 1 && of32.present == 0){
        return of31;

    }
    if(of31.present == 0 && of32.present == 1){
        return of32;
    }
    float d1 = length(of31.value-ro);
    float d2 = length(of32.value-ro);
    if(d2 < d1){
        return of32;
    }
    return of31;
}

OptFloat3 ray_box_intersection(float3 ro, float3 rd, float3 O, float3 radii){

    float3 O_EAST = O+float3_scaled_by(FLOAT3_EAST, radii.x);
    float3 O_NORTH = O+float3_scaled_by(FLOAT3_NORTH, radii.z);
    float3 O_WEST = O+float3_scaled_by(FLOAT3_WEST,radii.x);
    float3 O_SOUTH = O+float3_scaled_by(FLOAT3_SOUTH, radii.z);
    float3 O_UP = O+float3_scaled_by(FLOAT3_UP, radii.y);
    float3 O_DOWN = O+float3_scaled_by(FLOAT3_DOWN, radii.y);



    OptFloat3 of3_east = ray_panel_intersection(ro,rd,O_EAST,FLOAT3_EAST, FLOAT3_NORTH, FLOAT3_UP, radii.z, radii.y);
    OptFloat3 of3_north = ray_panel_intersection(ro,rd,O_NORTH,FLOAT3_NORTH, FLOAT3_WEST, FLOAT3_UP, radii.x, radii.y);
    OptFloat3 of3_west = ray_panel_intersection(ro,rd,O_WEST,FLOAT3_WEST, FLOAT3_SOUTH, FLOAT3_UP, radii.z, radii.y);
    OptFloat3 of3_south = ray_panel_intersection(ro,rd,O_SOUTH,FLOAT3_SOUTH, FLOAT3_EAST, FLOAT3_UP, radii.x, radii.y);
    OptFloat3 of3_up = ray_panel_intersection(ro,rd,O_UP, FLOAT3_UP, FLOAT3_EAST, FLOAT3_NORTH, radii.x, radii.z);
    OptFloat3 of3_down = ray_panel_intersection(ro,rd,O_DOWN, FLOAT3_DOWN, FLOAT3_EAST, FLOAT3_NORTH, radii.x, radii.z);

    OptFloat3 result = OptFloat3_miss();

    result = minimum_distance_union(result, of3_east, ro);
    result = minimum_distance_union(result, of3_north, ro);
    result = minimum_distance_union(result, of3_west, ro);
    result = minimum_distance_union(result, of3_south, ro);
    result = minimum_distance_union(result, of3_up, ro);
    result = minimum_distance_union(result, of3_down, ro);

    return result;

}


"#;