use glow::HasContext;
use glutin::{event_loop::EventLoop, window::WindowBuilder};

unsafe fn check() {
    let windowed_context_fresh = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 2)))
        .build_windowed(WindowBuilder::new(), &EventLoop::new())
        .unwrap();
    let windowed_context = windowed_context_fresh.make_current().unwrap();
    let gl =
        glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _);

    println!("Creating and initializing the buffers");
    let in_data = gl.create_buffer().unwrap();
    let out_data = gl.create_buffer().unwrap();
    initialize_buffers(&gl, in_data, out_data);

    println!("Loading the shader");
    let shader = gl.create_shader(glow::COMPUTE_SHADER).unwrap();
    gl.shader_source(shader, include_str!("shader.comp"));
    gl.compile_shader(shader);
    if !gl.get_shader_compile_status(shader) {
        panic!("Compilation failed: {}", gl.get_shader_info_log(shader));
    }

    let program = gl.create_program().unwrap();
    gl.attach_shader(program, shader);
    gl.link_program(program);
    gl.delete_shader(shader);
    if !gl.get_program_link_status(program) {
        panic!("Link failed: {}", gl.get_program_info_log(program));
    }
    assert_eq!(gl.get_error(), 0);

    println!("Active uniforms:");
    let count = gl.get_active_uniforms(program);
    for uniform in 0..count {
        print!("\t[{}]: ", uniform);
        let glow::ActiveUniform { size, utype, name } =
            gl.get_active_uniform(program, uniform).unwrap();
        println!("'{}' of {:?}x{}", name, utype, size);
    }
    assert_eq!(gl.get_error(), 0);

    println!("Compute pass");
    gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);
    gl.use_program(Some(program));
    gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 0, Some(in_data));
    gl.bind_buffer_base(glow::SHADER_STORAGE_BUFFER, 1, Some(out_data));
    assert_eq!(gl.get_error(), 0);

    gl.dispatch_compute(1, 1, 1);
    assert_eq!(gl.get_error(), 0);
    gl.memory_barrier(glow::SHADER_STORAGE_BARRIER_BIT);

    println!("Printing data after compute pass:");
    read_buffer_contents(&gl, out_data);
    assert_eq!(gl.get_error(), 0);
}

unsafe fn initialize_buffers(gl: &glow::Context, in_data: glow::Buffer, out_data: glow::Buffer) {
    let init_data: Vec<f32> = vec![-1.0; 64];
    let init_data_u8: &[u8] = core::slice::from_raw_parts(
        init_data.as_ptr() as *const u8,
        init_data.len() * core::mem::size_of::<f32>(),
    );

    gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(in_data));
    gl.buffer_data_u8_slice(
        glow::SHADER_STORAGE_BUFFER,
        init_data_u8,
        glow::DYNAMIC_COPY,
    );
    assert_eq!(gl.get_error(), 0);

    gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(out_data));
    gl.buffer_data_u8_slice(
        glow::SHADER_STORAGE_BUFFER,
        init_data_u8,
        glow::DYNAMIC_READ,
    );
    gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, None);
    assert_eq!(gl.get_error(), 0);
}

unsafe fn read_buffer_contents(gl: &glow::Context, buffer: glow::Buffer) {
    let _read_data: Vec<f32> = vec![42.0; 64];
    gl.memory_barrier(glow::BUFFER_UPDATE_BARRIER_BIT);
    gl.bind_buffer(glow::SHADER_STORAGE_BUFFER, Some(buffer));
    let mapped_buffer = gl.map_buffer_range(
        glow::SHADER_STORAGE_BUFFER,
        0,
        64 * core::mem::size_of::<f32>() as i32,
        glow::MAP_READ_BIT
        );
    assert_eq!(gl.get_error(), 0);
    let read_data_u8: &mut [f32] = core::slice::from_raw_parts_mut(
        mapped_buffer as *mut f32,
        64,
    );

    dbg!(read_data_u8);
    gl.unmap_buffer(glow::SHADER_STORAGE_BUFFER);
}

fn main() {
    unsafe { check() };
}
