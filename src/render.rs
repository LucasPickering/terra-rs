use crate::{
    world::{HasHexPosition, HexPointMap, Tile, TileLens, World},
    WorldConfig,
};
use cgmath::{EuclideanSpace, Matrix4, Point3, Rad, Vector3};
use log::debug;
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
use web_sys::KeyboardEvent;

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

const VERTEX_COLOR: VertexColor = VertexColor::new([0, 0, 255]);
const HEX_VERTICES: &[Vertex] = &[
    // 7 vertices make up the bottom face (center plus 6 outer vertices)
    Vertex::new(VertexPosition::new([0.0, 0.0, 0.0]), VERTEX_COLOR),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH, 0.0, 0.0]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([
            -TILE_SIDE_LENGTH / 2.0,
            0.0,
            -TILE_INSIDE_RADIUS,
        ]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH, 0.0, 0.0]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
    // 7 vertices make up the top face as well
    Vertex::new(VertexPosition::new([0.0, 1.0, 0.0]), VERTEX_COLOR),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH, 1.0, 0.0]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([TILE_SIDE_LENGTH / 2.0, 1.0, -TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([
            -TILE_SIDE_LENGTH / 2.0,
            1.0,
            -TILE_INSIDE_RADIUS,
        ]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH, 1.0, 0.0]),
        VERTEX_COLOR,
    ),
    Vertex::new(
        VertexPosition::new([-TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS]),
        VERTEX_COLOR,
    ),
];

/// A list of indices into the above vertex array. In this order, these vertices
/// define a hexagonal prism.
const HEX_INDICES: &[u8] = &[
    // top face
    2, 1, 0, //
    3, 2, 0, //
    4, 3, 0, //
    5, 4, 0, //
    6, 5, 0, //
    1, 6, 0, //
    // Side 1
    1, 2, 8, //
    2, 9, 8, //
    // Side 2
    2, 3, 9, //
    3, 10, 9, //
    // Side 3
    3, 4, 10, //
    4, 11, 10, //
    // Side 4
    4, 5, 11, //
    5, 12, 11, //
    // Side 5
    5, 6, 12, //
    6, 13, 12, //
    // Side 6
    6, 1, 13, //
    1, 8, 13, //
    // Top face
    7, 8, 9, //
    7, 9, 10, //
    7, 10, 11, //
    7, 11, 12, //
    7, 12, 13, //
    7, 13, 8, //
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

#[wasm_bindgen]
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

        let projection = cgmath::perspective(FOVY, 4.89, Z_NEAR, Z_FAR);
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
            view,
        }
    }

    #[wasm_bindgen]
    pub fn handle_keyboard_event(&mut self, event: KeyboardEvent) {
        // let event: KeyboardEvent = event.into();
        debug!(
            "{:?}; char_code={}; key_code={}",
            event,
            event.char_code(),
            event.key_code()
        );
    }

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
                &PipelineState::default().set_clear_color([0.5, 0.5, 0.5, 1.0]),
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
