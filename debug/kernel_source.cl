

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



    #define CAMERA_Z -10.0
    #define F 1.0
    #define MAX_MARCH_STEPS 1024
    #define SURFACE_EPSILON 0.05

    typedef struct VolumeData {
        int enabled;
        float3 radius;
    } VolumeData;

    VolumeData vd_build(float * buffer){
        VolumeData data;
        data.enabled = (buffer[0] == 1.0);
        float3 radius = (float3)(0.5,0.5,0.5);
        if(data.enabled){

        }
        data.radius = radius;
        return data;
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

    float volume_boundary_sdf(float3 world_coords, ApplicationState application_state){
        float3 local_coords = world_to_local_coords(world_coords, application_state);
        local_coords = fabs(local_coords);
        float3 delta = local_coords - application_state.vd->radius;
        return max3(delta.x, delta.y, delta.z);
    }

    float in_volume_sdf(float3 world_coords, ApplicationState application_state){
        return -1.0; // not yet implemented
    }

    float sdf(float3 world_coords, ApplicationState application_state){
        float coarse_sdf = volume_boundary_sdf(world_coords, application_state);
        if (coarse_sdf > SURFACE_EPSILON){
            return coarse_sdf;
        }else{
            if(application_state.vd->enabled){
                return in_volume_sdf(world_coords, application_state);
            }else{
                return coarse_sdf;
            }
        }
        return coarse_sdf;
    }

    float march(float3 ro, float3 rd, ApplicationState application_state){
        float3 pt = ro;
        for(int i=0;i<MAX_MARCH_STEPS;i++){
            float d =  sdf(pt, application_state);
            if((pt.z>0)&&(d > length(application_state.vd->radius))){
                return -1.0;
            }
            if( d <= SURFACE_EPSILON){
                return length(pt-ro);
            }else{
                pt += float3_scaled_by(rd,d);
            }
        }
        return -1.0;
    }

    Color shade(float3 pt, ApplicationState application_state){
        return color_from_float3((float3)(1.0,1.0,1.0));
    }

    __kernel void render(__global int * screen_dimensions, __global float * screen_buffer, __global float * input_data_buffer, __global float * axes_buffer){

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
 
        int y = id / screen_dimensions[1];
        int x = id % screen_dimensions[0];

        int w = screen_dimensions[0];
        int h = screen_dimensions[1];

        float u = (float) x / (float) w;
        float v = (float) y / (float) h;

        u = u - 0.5;
        v = 0.5 - v;

        u = u * (float)w / (float)h; 

        Color color = color_new(u,v,0.0);

        float3 ro = (float3)(0.0,0.0,CAMERA_Z);
        float3 rd = normalize((float3)(u,v,F));

        float dist = march(ro, rd, application_state);

        if(dist > 0.0){
            float3 pt = ro + float3_scaled_by(rd,dist);
            color = shade(pt, application_state);
        }

        int tid = y * w + x;
        int offs = tid * 3;
        screen_buffer[offs+0] = color.r;
        screen_buffer[offs+1] = color.g;
        screen_buffer[offs+2] = color.b;

    }

