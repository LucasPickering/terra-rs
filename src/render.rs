use crate::{
    world::{HasHexPosition, HexPointMap, Tile, TileLens, World},
    WorldConfig,
};
use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use luminance::{shader::Uniform, Semantics, UniformInterface, Vertex};
use luminance_front::{
    context::GraphicsContext as _,
    pipeline::PipelineState,
    render_state::RenderState,
    shader::Program,
    tess::{Interleaved, Mode, Tess},
};
use luminance_web_sys::WebSysWebGL2Surface;
use wasm_bindgen::prelude::*;

// We get the shader at compile time from local files
const VS: &str = include_str!("./shaders/simple-vs.glsl");
const FS: &str = include_str!("./shaders/simple-fs.glsl");

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 10.;

const TILE_SIDE_LENGTH: f32 = 1.0;
const TILE_INSIDE_RADIUS: f32 = TILE_SIDE_LENGTH * 0.866_025; // approx sqrt(3)/2
const TILE_WIDTH: f32 = TILE_SIDE_LENGTH * 2.0;
const TILE_MESH_NAME: &str = "tile";

// Vertex semantics. Those are needed to instruct the GPU how to select vertex’s
// attributes from the memory we fill at render time, in shaders. You don’t have
// to worry about them; just keep in mind they’re mandatory and act as
// “protocol” between GPU’s memory regions and shaders.
//
// We derive Semantics automatically and provide the mapping as field
// attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Semantics)]
pub enum VertexSemantics {
    //
    // - Reference vertex positions with the "position" variable in vertex
    //   shaders.
    // - The underlying representation is [f32; 3], which is a vec3 in GLSL.
    // - The wrapper type you can use to handle such a semantics is
    //   VertexPosition.
    #[sem(name = "position", repr = "[f32; 3]", wrapper = "VertexPosition")]
    Position,
    //
    // - Reference vertex colors with the "color" variable in vertex shaders.
    // - The underlying representation is [u8; 3], which is a uvec3 in GLSL.
    // - The wrapper type you can use to handle such a semantics is
    //   VertexColor.
    #[sem(name = "color", repr = "[u8; 3]", wrapper = "VertexColor")]
    Color,
}

// Our vertex type.
//
// We derive the Vertex trait automatically and we associate to each field the
// semantics that must be used on the GPU. The proc-macro derive Vertex will
// make sure for us every field we use has a mapping to the type you specified
// as semantics.
//
// Currently, we need to use #[repr(C))] to ensure Rust is not going to move
// struct’s fields around.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Vertex)]
#[vertex(sem = "VertexSemantics")]
struct Vertex {
    pos: VertexPosition,
    // Here, we can use the special normalized = <bool> construct to state
    // whether we want integral vertex attributes to be available as
    // normalized floats in the shaders, when fetching them from the vertex
    // buffers. If you set it to "false" or ignore it, you will get
    // non-normalized integer values (i.e. value ranging from 0 to 255 for
    // u8, for instance).
    #[vertex(normalized = "true")]
    rgb: VertexColor,
}

#[derive(Debug, UniformInterface)]
struct ShaderInterface {
    #[uniform(unbound)]
    projection: Uniform<[[f32; 4]; 4]>,
    #[uniform(unbound)]
    view: Uniform<[[f32; 4]; 4]>,
}

// // The vertices. We define two triangles.
// const TRI_VERTICES: [Vertex; 6] = [
//     // First triangle – an RGB one.
//     Vertex::new(
//         VertexPosition::new([0.5, -0.5]),
//         VertexColor::new([0, 255, 0]),
//     ),
//     Vertex::new(
//         VertexPosition::new([0.0, 0.5]),
//         VertexColor::new([0, 0, 255]),
//     ),
//     Vertex::new(
//         VertexPosition::new([-0.5, -0.5]),
//         VertexColor::new([255, 0, 0]),
//     ),
//     // Second triangle, a purple one, positioned differently.
//     Vertex::new(
//         VertexPosition::new([-0.5, 0.5]),
//         VertexColor::new([255, 51, 255]),
//     ),
//     Vertex::new(
//         VertexPosition::new([0.0, -0.5]),
//         VertexColor::new([51, 255, 255]),
//     ),
//     Vertex::new(
//         VertexPosition::new([0.5, 0.5]),
//         VertexColor::new([51, 51, 255]),
//     ),
// ];

// // The vertices, deinterleaved versions. We still define two triangles.
// const TRI_DEINT_POS_VERTICES: &[VertexPosition] = &[
//     VertexPosition::new([0.5, -0.5]),
//     VertexPosition::new([0.0, 0.5]),
//     VertexPosition::new([-0.5, -0.5]),
//     VertexPosition::new([-0.5, 0.5]),
//     VertexPosition::new([0.0, -0.5]),
//     VertexPosition::new([0.5, 0.5]),
// ];

// const TRI_DEINT_COLOR_VERTICES: &[VertexColor] = &[
//     VertexColor::new([0, 255, 0]),
//     VertexColor::new([0, 0, 255]),
//     VertexColor::new([255, 0, 0]),
//     VertexColor::new([255, 51, 255]),
//     VertexColor::new([51, 255, 255]),
//     VertexColor::new([51, 51, 255]),
// ];

// // Indices into TRI_VERTICES to use to build up the triangles.
// const TRI_INDICES: [u8; 6] = [
//     0, 1, 2, // First triangle.
//     3, 4, 5, // Second triangle.
// ];

const HEX_VERTICES: &[Vertex] = &[
    Vertex::new(
        VertexPosition::new([0.0, 0.0, 0.0]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH, 0.0, 0.0]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([
            -TILE_SIDE_LENGTH / 2.0,
            0.0,
            -TILE_INSIDE_RADIUS,
        ]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH, 0.0, 0.0]),
        VertexColor::new([0, 0, 255]),
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS]),
        VertexColor::new([0, 0, 255]),
    ),
];

const HEX_INDICES: &[u8] = &[
    2, 1, 0, // center
    3, 2, 0, // TODO
    4, 3, 0, // TODO
    5, 4, 0, // TODO
    6, 5, 0, // TODO
    1, 6, 0, // TODO
];

/// A convenient type to return as opaque to JS.
#[wasm_bindgen]
pub struct Scene {
    surface: WebSysWebGL2Surface,
    program: Program<VertexSemantics, (), ShaderInterface>,
    projection: Matrix4<f32>,
    view: Matrix4<f32>,
    hexagon: Tess<Vertex, u8, (), Interleaved>,
}

impl Scene {
    pub fn new(canvas_id: &str) -> Scene {
        // First thing first: we create a new surface to render to and get
        // events from.
        let mut surface =
            WebSysWebGL2Surface::new(canvas_id).expect("web-sys surface");

        // We need a program to “shade” our triangles and to tell luminance
        // which is the input vertex type, and we’re not interested in
        // the other two type variables for this sample.
        let program = surface
            .new_shader_program::<VertexSemantics, (), ShaderInterface>()
            .from_strings(VS, None, None, FS)
            .expect("program creation")
            .ignore_warnings();

        let hexagon = surface
            .new_tess()
            .set_vertices(HEX_VERTICES)
            .set_indices(HEX_INDICES)
            .set_mode(Mode::TriangleFan)
            .build()
            .unwrap();

        let projection = cgmath::perspective(FOVY, 1.0, Z_NEAR, Z_FAR);
        let view = Matrix4::<f32>::look_at(
            Point3::new(2., 2., 2.),
            Point3::origin(),
            Vector3::unit_y(),
        );

        Scene {
            surface,
            program,
            hexagon,
            projection,
            view, /* direct_triangles,
                   * indexed_triangles,
                   * direct_deinterleaved_triangles,
                   * indexed_deinterleaved_triangles, */
        }
    }
}

#[wasm_bindgen]
impl Scene {
    #[wasm_bindgen]
    pub fn render(&mut self) {
        let back_buffer = self.surface.back_buffer().unwrap();
        let Self {
            ref mut program,
            ref hexagon,
            ref projection,
            ref view,
            ..
        } = self;

        // Create a new dynamic pipeline that will render to the back buffer and
        // must clear it with pitch black prior to do any render to it.
        self.surface
            .new_pipeline_gate()
            .pipeline(
                &back_buffer,
                &PipelineState::default(),
                |_, mut shd_gate| {
                    shd_gate.shade(program, |mut iface, uni, mut rdr_gate| {
                        iface.set(&uni.projection, (*projection).into());
                        iface.set(&uni.view, (*view).into());

                        rdr_gate
                            .render(&RenderState::default(), |mut tess_gate| {
                                tess_gate.render(hexagon)
                            })
                    })
                },
            )
            .assume()
            .into_result()
            .unwrap()
    }
}
