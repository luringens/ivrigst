use crate::{
    render_gl::{
        self, buffer,
        data::{self, f32_f32_f32},
        Program,
    },
    resources::Resources,
};
use anyhow::{Context, Result};
use render_gl_derive::VertexAttribPointers;

#[derive(Copy, Clone, Debug, VertexAttribPointers)]
#[repr(C, packed)]
struct Vertex {
    #[location = 0]
    pos: data::f32_f32_f32,
    #[location = 1]
    normal: data::f32_f32_f32,
}

pub struct Model {
    program: render_gl::Program,
    vao: buffer::VertexArray,
    _vbo: buffer::ArrayBuffer,
    ibo: buffer::ElementArrayBuffer,
    indices: i32,
}

impl Model {
    pub fn new(res: &Resources) -> Result<Self> {
        // set up shader program
        let program = render_gl::Program::from_res(res, "shaders/triangle")?;

        let model = res.load_model("box.obj").context("Failed to load model.")?;

        // Set up vertex buffer object
        dbg!(&model.positions.len());
        dbg!(&model.normals.len());
        dbg!(&model.indices.len());

        // let indices = vec![
        //     0, 1, 3, 0, 3, 2, 3, 1, 4, 3, 4, 2, 0, 5, 7, 0, 7, 6, 7, 5, 8, 7, 8, 6, 0, 9, 11, 0,
        //     11, 10, 11, 9, 12, 11, 12, 10, 0, 13, 15, 0, 15, 14, 15, 13, 16, 15, 16, 14,
        // ];

        let vertices: Vec<Vertex> = model
            .positions
            .chunks_exact(3)
            .zip(model.normals.chunks_exact(3))
            .map(|(p, n)| {
                (
                    f32_f32_f32::from((p[0], p[1], p[2])),
                    f32_f32_f32::from((n[0], n[1], n[2])),
                )
            })
            .map(|(pos, normal)| Vertex { pos, normal })
            .collect();

        /*#[rustfmt::skip]
        let vertices = vec![
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.583,  0.771,  0.014).into() },
            Vertex { pos: (-1.0,-1.0, 1.0).into(), normal: (0.609,  0.115,  0.436).into() },
            Vertex { pos: (-1.0, 1.0, 1.0).into(), normal: (0.327,  0.483,  0.844).into() },
            Vertex { pos: (1.0, 1.0,-1.0).into(), normal: (0.822,  0.569,  0.201).into() },
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.435,  0.602,  0.223).into() },
            Vertex { pos: (-1.0, 1.0,-1.0).into(), normal: (0.310,  0.747,  0.185).into() },
            Vertex { pos: (1.0,-1.0, 1.0).into(), normal: (0.597,  0.770,  0.761).into() },
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.559,  0.436,  0.730).into() },
            Vertex { pos: (1.0,-1.0,-1.0).into(), normal: (0.359,  0.583,  0.152).into() },
            Vertex { pos: (1.0, 1.0,-1.0).into(), normal: (0.483,  0.596,  0.789).into() },
            Vertex { pos: (1.0,-1.0,-1.0).into(), normal: (0.559,  0.861,  0.639).into() },
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.195,  0.548,  0.859).into() },
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.014,  0.184,  0.576).into() },
            Vertex { pos: (-1.0, 1.0, 1.0).into(), normal: (0.771,  0.328,  0.970).into() },
            Vertex { pos: (-1.0, 1.0,-1.0).into(), normal: (0.406,  0.615,  0.116).into() },
            Vertex { pos: (1.0,-1.0, 1.0).into(), normal: (0.676,  0.977,  0.133).into() },
            Vertex { pos: (-1.0,-1.0, 1.0).into(), normal: (0.971,  0.572,  0.833).into() },
            Vertex { pos: (-1.0,-1.0,-1.0).into(), normal: (0.140,  0.616,  0.489).into() },
            Vertex { pos: (-1.0, 1.0, 1.0).into(), normal: (0.997,  0.513,  0.064).into() },
            Vertex { pos: (-1.0,-1.0, 1.0).into(), normal: (0.945,  0.719,  0.592).into() },
            Vertex { pos: (1.0,-1.0, 1.0).into(), normal: (0.543,  0.021,  0.978).into() },
            Vertex { pos: (1.0, 1.0, 1.0).into(), normal: (0.279,  0.317,  0.505).into() },
            Vertex { pos: (1.0,-1.0,-1.0).into(), normal: (0.167,  0.620,  0.077).into() },
            Vertex { pos: (1.0, 1.0,-1.0).into(), normal: (0.347,  0.857,  0.137).into() },
            Vertex { pos: (1.0,-1.0,-1.0).into(), normal: (0.055,  0.953,  0.042).into() },
            Vertex { pos: (1.0, 1.0, 1.0).into(), normal: (0.714,  0.505,  0.345).into() },
            Vertex { pos: (1.0,-1.0, 1.0).into(), normal: (0.783,  0.290,  0.734).into() },
            Vertex { pos: (1.0, 1.0, 1.0).into(), normal: (0.722,  0.645,  0.174).into() },
            Vertex { pos: (1.0, 1.0,-1.0).into(), normal: (0.302,  0.455,  0.848).into() },
            Vertex { pos: (-1.0, 1.0,-1.0).into(), normal: (0.225,  0.587,  0.040).into() },
            Vertex { pos: (1.0, 1.0, 1.0).into(), normal: (0.517,  0.713,  0.338).into() },
            Vertex { pos: (-1.0, 1.0,-1.0).into(), normal: (0.053,  0.959,  0.120).into() },
            Vertex { pos: (-1.0, 1.0, 1.0).into(), normal: (0.393,  0.621,  0.362).into() },
            Vertex { pos: (1.0, 1.0, 1.0).into(), normal: (0.673,  0.211,  0.457).into() },
            Vertex { pos: (-1.0, 1.0, 1.0).into(), normal: (0.820,  0.883,  0.371).into() },
            Vertex { pos: (1.0,-1.0, 1.0).into(), normal: (0.982,  0.099,  0.879).into() },
        ];*/

        let vbo = buffer::ArrayBuffer::new();
        vbo.bind();
        vbo.static_draw_data(&vertices);

        // set up vertex array object
        let vao = buffer::VertexArray::new();
        vao.bind();
        Vertex::vertex_attrib_pointers();

        // indices buffer
        let ibo = buffer::ElementArrayBuffer::new();
        ibo.bind();
        ibo.static_draw_data(&model.indices);
        ibo.unbind();
        vbo.unbind();
        vao.unbind();

        // panic!();

        Ok(Self {
            program,
            _vbo: vbo,
            vao,
            ibo,
            indices: model.indices.len() as i32,
        })
    }

    pub fn shader(&mut self) -> &mut Program {
        &mut self.program
    }

    pub fn render(&self) {
        self.program.set_used();
        self.vao.bind();
        self.ibo.bind();

        unsafe {
            gl::DrawElements(
                gl::TRIANGLES,
                self.indices,
                gl::UNSIGNED_INT,             // starting index in the enabled arrays
                0 as *const std::ffi::c_void, // number of indices to be rendered
            );
        }
        self.ibo.unbind();
        self.vao.unbind();
    }
}
