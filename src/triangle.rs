use crate::{
    render_gl::{self, buffer, data},
    resources::Resources,
};
use anyhow::Result;
use render_gl_derive::VertexAttribPointers;

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    clr: data::f32_f32_f32,
}

pub struct Triangle {
    program: render_gl::Program,
    vao: buffer::VertexArray,
    _vbo: buffer::ArrayBuffer,
}

impl Triangle {
    pub fn new(res: &Resources) -> Result<Triangle> {
        // set up shader program

        let program = render_gl::Program::from_res(res, "shaders/triangle")?;

        // set up vertex buffer object
        #[rustfmt::skip]
        let vertices: Vec<Vertex> = vec![
            Vertex { pos: ( 0.5, -0.5, 0.0).into(), clr: (1.0, 0.0, 0.0).into() }, // bottom right
            Vertex { pos: (-0.5, -0.5, 0.0).into(), clr: (0.0, 1.0, 0.0).into() }, // bottom left
            Vertex { pos: ( 0.0,  0.5, 0.0).into(), clr: (0.0, 0.0, 1.0).into() }  // top
        ];

        let vbo = buffer::ArrayBuffer::new();
        vbo.bind();
        vbo.static_draw_data(&vertices);
        vbo.unbind();

        // set up vertex array object

        let vao = buffer::VertexArray::new();

        vao.bind();
        vbo.bind();
        Vertex::vertex_attrib_pointers();
        vbo.unbind();
        vao.unbind();

        Ok(Triangle {
            program,
            _vbo: vbo,
            vao,
        })
    }

    pub fn render(&self) {
        self.program.set_used();
        self.vao.bind();

        unsafe {
            gl::DrawArrays(
                gl::TRIANGLES, // mode
                0,             // starting index in the enabled arrays
                3,             // number of indices to be rendered
            );
        }
    }
}
