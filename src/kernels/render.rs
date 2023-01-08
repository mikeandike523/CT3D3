pub const RENDER: &str = r#"



    #define F 1.0
    #define FIXED_STEP_MARCH_ENTER_MAX_STEPS 32
    #define DOWNSAMPLING 1

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

        float3 color = (float3)(u,v,0.0);

        float camera_z = general_parameters_buffer[0];

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

                    float total = 0.0;
                    float denom = 0.0;
    
                    while(vd_float3_is_in_bounds(&vd, local_pt)&&(length(local_pt)<max_distance))
                    {
                        float value = vd_query(&vd, local_pt);
                        if(value > 0.0){
                            total += value;
                            denom += 1.0;
                        }
                        local_pt += float3_scaled_by(local_dir,fixed_march_step);
                    }
    
                    if(denom > 0.0){
                        float grey = total / denom;
                        color = (float3)(grey, grey, grey);
                    }

                }


            }else{
                color=(float3)(0.0,0.0,1.0);
            }
        }else{
            // Do nothing
        }

        int tid = y * w + x;
        int offs = tid * 3;
        screen_buffer[offs+0] = color.x;
        screen_buffer[offs+1] = color.y;
        screen_buffer[offs+2] = color.z;

    }

"#;