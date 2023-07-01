use web_sys::{WebGlProgram, WebGl2RenderingContext, WebGlShader};

use crate::log_str;

pub fn create_program_from_src(gl:&WebGl2RenderingContext, vertex_src:&str, frag_src:&str) -> WebGlProgram {
    let vertex_shader = match compile_shader(
        gl, 
        WebGl2RenderingContext::VERTEX_SHADER,
        &vertex_src
    ) {
        Ok(v) => v,
        Err(s) => {
            log_str("Error when compiling vertex shader");
            log_str(&s);
            panic!();
        }
    };

    let frag_shader = match compile_shader(
        gl, 
        WebGl2RenderingContext::FRAGMENT_SHADER,
        &frag_src
    ) {
        Ok(v) => v,
        Err(s) => {
            log_str("Error when compiling fragment shader");
            log_str(&s);
            panic!();
        }
    };

    let program = match link_program(gl, &vertex_shader, &frag_shader) {
        Ok(v) => v,
        Err(s) => {
            log_str("Error linking program");
            log_str(&s);
            panic!();
        }
    };
    program
}

fn compile_shader(
    context: &WebGl2RenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

fn link_program(
    context: &WebGl2RenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}