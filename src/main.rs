use glow::HasContext as _;
use glutin::{event_loop::EventLoop, window::WindowBuilder};

unsafe fn check() {
    let windowed_context_fresh = glutin::ContextBuilder::new()
        .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGlEs, (3, 2)))
        .build_windowed(WindowBuilder::new(), &EventLoop::new())
        .unwrap();
    let windowed_context = windowed_context_fresh.make_current().unwrap();
    let gl =
        glow::Context::from_loader_function(|s| windowed_context.get_proc_address(s) as *const _);

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
}

fn main() {
    unsafe { check() };
}
