pub const RENDER: &str = r#"

    __kernel void render(__global int * screen_dimensions, __global float * screen_buffer){
        
        int id = get_global_id(0);
 
        int y = id / screen_dimensions[1];
        int x = id % screen_dimensions[0];

        int w = screen_dimensions[0];
        int h = screen_dimensions[1];

        float u = (float) x / (float) w;
        float v = (float) y / (float) h;

        int tid = y * w + x;

        int offs = tid * 3;

        screen_buffer[offs+0] = u;
        screen_buffer[offs+1] = v;
        screen_buffer[offs+2] = 0;

    }

"#;