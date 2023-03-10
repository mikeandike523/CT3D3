
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

#define F 1.0
#define FIXED_STEP_MARCH_ENTER_MAX_STEPS 32
#define DOWNSAMPLING 1
#define NORMAL_SEARCH_RADIUS 1
#define DROPOFF_RATE 0.70
#define INITIAL_SCALE 1.05


typedef struct VolumeData {
    int enabled;
    float3 radii;
    int3 res;
    float * buffer;
} VolumeData;

VolumeData vd_build(float * buffer){
    VolumeData data;
    data.buffer = buffer;
    data.enabled = (buffer[0] == 1.0);
    float3 radii = (float3)(0.5,0.5,0.5);
    int3 res = (int3)(0,0,0);
    if(data.enabled){

        // Data Layout

        // POSITION 0: enabled/disabled
        // POSITION 1-3: radii
        // POSITION 4-6: resolution for each (whole) size, not just the radii
        //               if the resolution is odd, the first cell will tend to the negative side (i.e. floor). 
        //               Thus the error will be -0.5 cells.


        // The value inside each cell is betwen 0.0 and 1.0, and represents the density at a given point.
        // This will NOT be clipped or error checked on the GPU side
        // Since only one volume will be present, there is an implied "view" to `buffer` starting from position 8 and onward.


        radii.x = buffer[1];
        radii.y = buffer[2];
        radii.z = buffer[3];

        int idx = 4;

        res.x = (int)buffer[idx++];
        res.y = (int)buffer[idx++];
        res.z = (int)buffer[idx++];

    }
    data.radii = radii;
    data.res = res;
    return data;
}

float vd_get_by_int3(VolumeData* vd, int3 coord){
    int header_start = 7;
    int W = vd->res.x;
    int H = vd->res.y;
    int D = vd->res.z;
    int id = W*H*coord.z + W * coord.y + coord.x;
    return (float)vd->buffer[header_start+id];
}

float vd_float3_is_in_bounds(VolumeData* vd, float3 coord){
    float3 radii = vd->radii;
    return coord.x >= -radii.x && coord.x < radii.x && coord.y >= -radii.y && coord.y < radii.y && coord.z >= -radii.z && coord.z < radii.z;
}

int3 vd_map_float3(VolumeData* vd, float3 coord){

    float u = (coord.x + vd->radii.x) / (2.0 * vd->radii.x);
    float v = (coord.y + vd->radii.y) / (2.0 * vd->radii.y);
    float w = (coord.z + vd->radii.z) / (2.0 * vd->radii.z);

    int3 id3 = (int3)(
        (int)(u*(float)vd->res.x),
        (int)(v*(float)vd->res.y),
        (int)(w*(float)vd->res.z)
    );

    return id3;

}

float vd_query(VolumeData * vd, float3 coord){
    if (!vd_float3_is_in_bounds(vd, coord)){
        return -1.0;
    }
    int3 icoord = vd_map_float3(vd, coord);
    return vd_get_by_int3(vd, icoord);
}

float vd_get_march_step(VolumeData * vd){


    float3 radii = vd->radii;
    float3 cell_size = (float3)(
        (2.0*radii.x)/(float)vd->res.x,
        (2.0*radii.y)/(float)vd->res.y,
        (2.0*radii.z)/(float)vd->res.z
    );
    return min3(cell_size.x, cell_size.y, cell_size.z);
}

float3 vd_get_normal(VolumeData * vd, float3 coord, float low_cutoff){

    float3 radii = vd->radii;

    float3 cell_sizes = (float3)(
        (2.0*radii.x)/(float)vd->res.x,
        (2.0*radii.y)/(float)vd->res.y,
        (2.0*radii.z)/(float)vd->res.z
    );

    float3 total_normal = (float3)(0.0, 0.0, 0.0);

    for(int dx = -NORMAL_SEARCH_RADIUS; dx <= NORMAL_SEARCH_RADIUS; dx++){
        for(int dy = -NORMAL_SEARCH_RADIUS; dy <= NORMAL_SEARCH_RADIUS;dy++){
            for(int dz = -NORMAL_SEARCH_RADIUS; dz <= NORMAL_SEARCH_RADIUS;dz++){
                float3 offs = towards_east((float)dx*cell_sizes.x) + towards_north((float)dz*cell_sizes.z) + towards_up((float)dy*cell_sizes.y);
                float value = vd_query(vd, coord+offs);
                total_normal -= value * offs;   
            }
        }
    }

    return safe_normalize(total_normal);

}

typedef struct ApplicationState {
    VolumeData * vd;
    float3 RIGHT;
    float3 UP;
    float3 FORWARD;        
} ApplicationState;

float3 world_to_local_coords(float3 world_coords, ApplicationState application_state){
    float3 local_coords;
    local_coords.x = dot(application_state.RIGHT, world_coords);
    local_coords.y = dot(application_state.UP, world_coords);
    local_coords.z = dot(application_state.FORWARD, world_coords);
    return local_coords;
}

float3 local_to_world_coords(float3 local_coords, ApplicationState application_state){
    return float3_scaled_by(application_state.RIGHT, local_coords.x) + float3_scaled_by(application_state.UP, local_coords.y) + float3_scaled_by(application_state.FORWARD, local_coords.z);
}

float volume_boundary_sdf(float3 world_coords, ApplicationState application_state){
    float3 local_coords = world_to_local_coords(world_coords, application_state);
    local_coords = fabs(local_coords);
    float3 delta = local_coords - application_state.vd->radii;
    return max3(delta.x, delta.y, delta.z);
}

float sdf(float3 world_coords, ApplicationState application_state){
    float coarse_sdf = volume_boundary_sdf(world_coords, application_state);
    return coarse_sdf;
}

// float march(float3 ro, float3 rd, ApplicationState application_state){
//     float3 pt = ro;
//     for(int i=0;i<FIND_START_POINT_MAX_MARCH_STEPS;i++){
//         float d =  sdf(pt, application_state);
//         if((pt.z>0)&&(d > length(application_state.vd->radii))){
//             return -1.0;
//         }
//         if( d <= SURFACE_EPSILON){
//             return length(pt-ro);
//         }else{
//             pt += float3_scaled_by(rd,d);
//         }
//     }
//     return -1.0;
// }

__kernel void render(
    __global int * screen_dimensions,
    __global float * screen_buffer,
    __global float * input_data_buffer,
    __global float * axes_buffer,
    __global float * general_parameters_buffer
){

    ApplicationState application_state;

    application_state.RIGHT = (float3)(
        axes_buffer[0*3+0],
        axes_buffer[0*3+1],
        axes_buffer[0*3+2]
    );

    application_state.UP = (float3)(
        axes_buffer[1*3+0],
        axes_buffer[1*3+1],
        axes_buffer[1*3+2]
    );

    application_state.FORWARD = (float3)(
        axes_buffer[2*3+0],
        axes_buffer[2*3+1],
        axes_buffer[2*3+2]
    );

    VolumeData vd = vd_build(input_data_buffer);

    application_state.vd = &vd;
    
    int id = get_global_id(0);

    int y = id / screen_dimensions[0];
    int x = id % screen_dimensions[0];

    int w = screen_dimensions[0];
    int h = screen_dimensions[1];

    float u = (float) x / (float) w;
    float v = (float) y / (float) h;

    u = u - 0.5;
    v = 0.5 - v;

    u = u * (float)w / (float)h; 

    float camera_z = general_parameters_buffer[0];

    float LOW_CUTOFF = general_parameters_buffer[1];

    float3 color = ((float)x/(float)w >= LOW_CUTOFF) ? ((float3)(1.0,1.0,1.0)) : ((float3)(0.0,0.0,0.0));

    float3 ro = (float3)(0.0,0.0,camera_z);
    float3 rd = normalize((float3)(u,v,F));

    float3 local_ro = world_to_local_coords(ro, application_state);

    float3 local_rd = world_to_local_coords(rd, application_state);

    OptFloat3 box_intersection = ray_box_intersection(local_ro, local_rd, (float3)(0.0,0.0,0.0),vd.radii);

    if(box_intersection.present==1){
        
        float fixed_march_step = vd_get_march_step(&vd) * ((float)DOWNSAMPLING);

        float3 local_start = box_intersection.value;

        float3 local_dir = world_to_local_coords(rd, application_state);

        float3 local_pt = local_start;

        if(vd.enabled){

            float max_distance = length(vd.radii);

            int step_counter = 0;

            // Step 1: step until the start point is inside
            while(!vd_float3_is_in_bounds(&vd, local_pt) && step_counter < FIXED_STEP_MARCH_ENTER_MAX_STEPS)
            {
                local_pt += float3_scaled_by(local_dir,fixed_march_step);
                step_counter+=1;
            }

            if(step_counter <= FIXED_STEP_MARCH_ENTER_MAX_STEPS){

                // Step 2: step until point exits the volume

                OptFloat3 ipoint = OptFloat3_miss();

                while(vd_float3_is_in_bounds(&vd, local_pt)&&(length(local_pt)<max_distance))
                {
                    float value = vd_query(&vd, local_pt);
                    if(value >= LOW_CUTOFF){
                        
                        ipoint = OptFloat3_hit(local_pt);
                        break;
                    }
                    local_pt += float3_scaled_by(local_dir,fixed_march_step);
                }

                if(ipoint.present){
                    float3 world_pt = local_to_world_coords(ipoint.value,application_state);
                    float grey = INITIAL_SCALE-DROPOFF_RATE*length(world_pt - (float3)(0.0,0.0,camera_z))/fabs(camera_z);
                    float _u = fabs(ipoint.value.x/vd.radii.x);
                    float _v = fabs(ipoint.value.y/vd.radii.y);
                    float _w = fabs(ipoint.value.z/vd.radii.z);
                    float a = 1.0 - 0.5 * _u - 0.5 * _v;
                    float b = 1.0 - 0.5 * _v - 0.5 * _w;
                    float c = 1.0 - 0.5 * _w - 0.5 * _u;
                    float3 base_color =(float3)(
                        a+_u,
                        b+_v,
                        c+_w
                    );
                    color = float3_scaled_by(base_color,grey);
                }

            }

        }else{
            
        }
    }else{

    }

    int tid = y * w + x;
    int offs = tid * 3;
    screen_buffer[offs+0] = color.x;
    screen_buffer[offs+1] = color.y;
    screen_buffer[offs+2] = color.z;

}
