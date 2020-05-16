use euclid::default::Transform3D;
extern crate gl;
// use super::*;

use gl::types::*;
use std::convert::AsRef;
use std::ffi::CString;
use std::fs;
use std::path::PathBuf;
use std::ptr;
use std::str;

use crate::renderer::RenderError;
use crate::renderer::Vector4D;

pub fn load_file<P: AsRef<std::path::Path>>(filename: P) -> String {
    let content = fs::read_to_string(&filename).expect(
        format!(
            "Failed to load file: '{}'",
            filename.as_ref().to_str().unwrap()
        )
        .as_str(),
    );
    content
}

macro_rules! gl_check {
    ($f:expr) => {{
        $f;
        if cfg!(debug_assertions) {
            let err = gl::GetError();
            // println!("Error {:?}", err);
            if err != gl::NO_ERROR {
                let err_str = match err {
                    gl::INVALID_ENUM => "GL_INVALID_ENUM",
                    gl::INVALID_VALUE => "GL_INVALID_VALUE",
                    gl::INVALID_OPERATION => "GL_INVALID_OPERATION",
                    gl::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
                    gl::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
                    gl::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
                    gl::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
                    _ => "unknown error",
                };

                panic!(
                    "{}:{} error {} {}",
                    file!(),
                    line!(),
                    std::stringify!($f),
                    err_str
                );
            }
        }
    }};
}

pub fn compile_shader(src: &str, ty: GLenum) -> GLuint {
    let shader;
    unsafe {
        shader = gl::CreateShader(ty);

        let c_str = CString::new(src.as_bytes()).unwrap();
        gl::ShaderSource(shader, 1, &c_str.as_ptr(), ptr::null());
        gl::CompileShader(shader);

        let mut status = gl::FALSE as GLint;

        if gl::GetShaderiv::is_loaded() {
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut status);
        } else {
            println!("GetShaderiv is not loaded");
        }
        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut len);

            let mut buf = Vec::with_capacity(len as usize);
            buf.set_len((len as usize) - 1);
            gl::GetShaderInfoLog(
                shader,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{:?} {}",
                ty,
                str::from_utf8(&buf)
                    .ok()
                    .expect("ShaderInfoLog not valid utf8")
            );
        }
    }
    shader
}

pub fn link_shader(vs: GLuint, fs: GLuint) -> GLuint {
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vs);
        gl::AttachShader(program, fs);

        gl::LinkProgram(program);

        let mut status = gl::FALSE as GLint;

        gl::GetProgramiv(program, gl::LINK_STATUS, &mut status);

        if status != (gl::TRUE as GLint) {
            let mut len = 0;
            gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut len);
            let mut buf = Vec::with_capacity(len as usize);
            //if len == 0 {
            //    println!("Status {} but len is 0", status);
            //    return program;
            //}
            buf.set_len((len as usize) - 1);
            gl::GetProgramInfoLog(
                program,
                len,
                ptr::null_mut(),
                buf.as_mut_ptr() as *mut GLchar,
            );

            panic!(
                "{}",
                str::from_utf8(&buf)
                    .ok()
                    .expect("LinkStatus not valid utf8")
            );
        }
        program
    }
}

pub trait Shader {
    type Object;

    fn bind(&self);

    fn unbind(&self) {
        unsafe {
            gl::UseProgram(0);
        }
    }

    fn name(&self) -> &str;

    fn bounded<P>(&self, functor: P)
        where P: FnOnce(&Self::Object);
}

pub struct RectShader {
    handle: u32,
    per_loc: i32,
    name: String,
}

impl RectShader {
    pub fn create() -> Self {

        Self {
            handle: 0,
            per_loc: 0,
            name: "rect".to_string(),
        }
    }

    pub fn init(&mut self) -> Result<(), RenderError> {
        let vs_src = load_file("./shaders/rect_vert.glsl");
        let fs_src = load_file("./shaders/rect_frag.glsl");

        let vertex = compile_shader(vs_src.as_str(), gl::VERTEX_SHADER);
        let fragment = compile_shader(fs_src.as_str(), gl::FRAGMENT_SHADER);
        let program = link_shader(vertex, fragment);

        let per_loc = unsafe {
            gl::UseProgram(program);
            let per_loc = gl::GetUniformLocation(program, super::to_c_str("perspective").as_ptr());
            gl::UseProgram(0);
            per_loc
        };

        self.handle = program;
        self.per_loc = per_loc as _;
        Ok(())
    }

    pub fn set_perspective(&self, perf: &Transform3D<f32>) {
        unsafe {
            gl_check!(gl::UniformMatrix4fv(
                self.per_loc as i32,
                1,
                gl::FALSE,
                perf.to_row_major_array().as_ptr() as *const _
            ))
        }
    }
}

impl Shader for RectShader {
    type Object = Self;

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn bind(&self) {
        unsafe {
            gl_check!(gl::UseProgram(self.handle));
        }
    }

    fn bounded<P>(&self, functor: P)
        where P: FnOnce(&Self::Object) {

        self.bind();
        functor(self);
        self.unbind();
    }
}

pub struct TextShader {
    handle: u32,
    per_loc: u32,
    tex_loc: u32,
    name: String,
}

impl TextShader {
    pub fn create() -> Self {
        Self {
            handle: 0,
            per_loc: 0,
            tex_loc: 0,
            name: "text".to_string(),
        }
    }

    pub fn init(&mut self) -> Result<(), RenderError> {

        let vs_src = load_file("./shaders/text_vert.glsl");
        let fs_src = load_file("./shaders/text_frag.glsl");

        let vertex = compile_shader(vs_src.as_str(), gl::VERTEX_SHADER);
        let fragment = compile_shader(fs_src.as_str(), gl::FRAGMENT_SHADER);
        let program = link_shader(vertex, fragment);

        let (per_loc, tex_loc) = unsafe {
            gl::UseProgram(program);
            let per_loc = gl::GetUniformLocation(program, super::to_c_str("perspective").as_ptr());
            let tex_loc = gl::GetUniformLocation(program, super::to_c_str("texs").as_ptr());
            gl::UseProgram(0);
            (per_loc, tex_loc)
        };

        self.handle = program;
        self.per_loc = per_loc as _;
        self.tex_loc = tex_loc as _;
        Ok(())
    }

    pub fn set_perspective(&self, perf: &Transform3D<f32>) {
        unsafe {
            gl_check!(gl::UniformMatrix4fv(
                self.per_loc as i32,
                1,
                gl::FALSE,
                perf.to_row_major_array().as_ptr() as *const _
            ));
        }
    }

    pub fn set_textures(&self, handle: &[u32]) {
        unsafe {
            gl_check!(gl::Uniform1iv(
                self.tex_loc as i32,
                handle.len() as _,
                handle.as_ptr() as *const _
            ));
        }
    }
}

impl Shader for TextShader {
    type Object = Self;

    fn name(&self) -> &str {
        self.name.as_str()
    }

    fn bind(&self) {
        unsafe {
            gl_check!(gl::UseProgram(self.handle));
        }
    }


    fn bounded<P>(&self, functor: P)
        where P: FnOnce(&Self::Object) {

        self.bind();
        functor(self);
        self.unbind();
    }
}
