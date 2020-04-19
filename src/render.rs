use crate::world::{
    HasHexPosition, HexPointMap, Tile, TileLens, World, WorldConfig,
};
use kiss3d::{
    camera::{ArcBall, Camera},
    event::{Action, Key, WindowEvent},
    light::Light,
    planar_camera::PlanarCamera,
    post_processing::PostProcessingEffect,
    renderer::Renderer,
    resource::{Mesh, MeshManager},
    scene::SceneNode,
    window::{State, Window},
};
use log::debug;
use nalgebra::{Point3, Translation3, Vector3};
use std::{cell::RefCell, rc::Rc};

const TILE_SIDE_LENGTH: f32 = 1.0;
const TILE_INSIDE_RADIUS: f32 = TILE_SIDE_LENGTH * 0.866_025; // approx sqrt(3)/2
const TILE_WIDTH: f32 = TILE_SIDE_LENGTH * 2.0;
const TILE_MESH_NAME: &str = "tile";

struct AppState {
    camera: Box<dyn Camera>,
    world: World,
    lens: TileLens,
    root_node: SceneNode,
    tile_nodes: HexPointMap<SceneNode>,
}

impl AppState {
    fn new(window: &mut Window) -> Self {
        let camera = ArcBall::new_with_frustrum(
            std::f32::consts::PI / 4.0,
            0.1,
            1024.0,
            Point3::new(-50.0, 50.0, -50.0),
            Point3::origin(),
        );

        let world_config = WorldConfig::load().unwrap();
        let world = World::generate(world_config);

        let mut root_node = window.add_group();
        let tile_nodes =
            render_tiles(&mut root_node, world.tiles(), TileLens::Composite);

        Self {
            camera: Box::new(camera),
            world,
            lens: TileLens::Composite,
            root_node,
            tile_nodes,
        }
    }

    fn handle_event(&mut self, window: &mut Window, event: &WindowEvent) {
        match event {
            WindowEvent::Key(Key::Key1, Action::Press, _) => {
                self.update_tile_color(TileLens::Composite);
            }
            WindowEvent::Key(Key::Key2, Action::Press, _) => {
                self.update_tile_color(TileLens::Elevation);
            }
            WindowEvent::Key(Key::Key3, Action::Press, _) => {
                self.update_tile_color(TileLens::Humidity);
            }
            WindowEvent::Key(Key::Key4, Action::Press, _) => {
                self.update_tile_color(TileLens::Biome);
            }
            WindowEvent::Key(Key::R, Action::Press, _) => {
                self.regenerate_world(window);
            }
            _ => {}
        }
    }

    fn regenerate_world(&mut self, window: &mut Window) {
        // Generate a new world
        let world_config = WorldConfig::load().unwrap();
        self.world = World::generate(world_config);

        // Swap out the nodes
        window.remove_node(&mut self.root_node);
        self.root_node = window.add_group();
        self.tile_nodes =
            render_tiles(&mut self.root_node, self.world.tiles(), self.lens);
    }

    fn update_tile_color(&mut self, lens: TileLens) {
        self.lens = lens;
        apply_tile_colors(self.world.tiles(), &mut self.tile_nodes, lens);
    }
}

impl State for AppState {
    fn step(&mut self, window: &mut Window) {
        for event in window.events().iter() {
            self.handle_event(window, &event.value);
        }
    }

    #[allow(clippy::type_complexity)]
    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (Some(self.camera.as_mut()), None, None, None)
    }
}

fn build_tile_mesh() -> Mesh {
    Mesh::new(
        vec![
            // Each of these starts at the center, then goes to the top-right,
            // then clockwise from there
            // Bottom
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS),
            Point3::new(TILE_SIDE_LENGTH, 0.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH, 0.0, 0.0),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS),
            // Top
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS),
            Point3::new(TILE_SIDE_LENGTH, 1.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 1.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 1.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH, 1.0, 0.0),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS),
        ],
        vec![
            // Bottom face
            Point3::new(2, 1, 0),
            Point3::new(3, 2, 0),
            Point3::new(4, 3, 0),
            Point3::new(5, 4, 0),
            Point3::new(6, 5, 0),
            Point3::new(1, 6, 0),
            // Side 1
            Point3::new(1, 2, 8),
            Point3::new(2, 9, 8),
            // Side 2
            Point3::new(2, 3, 9),
            Point3::new(3, 10, 9),
            // Side 3
            Point3::new(3, 4, 10),
            Point3::new(4, 11, 10),
            // Side 4
            Point3::new(4, 5, 11),
            Point3::new(5, 12, 11),
            // Side 5
            Point3::new(5, 6, 12),
            Point3::new(6, 13, 12),
            // Side 6
            Point3::new(6, 1, 13),
            Point3::new(1, 8, 13),
            // Top face
            Point3::new(7, 8, 9),
            Point3::new(7, 9, 10),
            Point3::new(7, 10, 11),
            Point3::new(7, 11, 12),
            Point3::new(7, 12, 13),
            Point3::new(7, 13, 8),
        ],
        None,
        None,
        false,
    )
}

fn init_meshes() {
    let mesh = Rc::new(RefCell::new(build_tile_mesh()));
    MeshManager::get_global_manager(move |mm| {
        mm.add(mesh.clone(), TILE_MESH_NAME)
    });
}

fn apply_tile_colors(
    tiles: &HexPointMap<Tile>,
    tile_nodes: &mut HexPointMap<SceneNode>,
    lens: TileLens,
) {
    for (pos, node) in tile_nodes.iter_mut() {
        let tile = tiles.get(pos).expect("Missing SceneNode for Tile");
        let color = tile.color(lens);
        node.set_color(color.red(), color.green(), color.blue());
    }
}

fn render_tiles(
    root_node: &mut SceneNode,
    tiles: &HexPointMap<Tile>,
    lens: TileLens,
) -> HexPointMap<SceneNode> {
    let mut tile_nodes: HexPointMap<_> = tiles
        .values()
        .map(|tile| (tile.position(), render_tile(root_node, tile)))
        .collect();
    apply_tile_colors(tiles, &mut tile_nodes, lens);
    tile_nodes
}

fn render_tile(parent: &mut SceneNode, tile: &Tile) -> SceneNode {
    let mut node = parent
        .add_geom_with_name(
            TILE_MESH_NAME,
            Vector3::new(
                1.0,
                (tile.elevation() - Tile::ELEVATION_RANGE.min) as f32,
                1.0,
            ),
        )
        .unwrap();

    node.enable_backface_culling(true);

    // Shift tile based on its position
    let translation: (f64, f64) =
        tile.position().get_pixel_pos(TILE_WIDTH as f64);
    node.set_local_translation(Translation3::new(
        translation.0 as f32,
        0.0,
        translation.1 as f32,
    ));

    node
}

pub fn run() {
    let mut window = Window::new("Terra");
    init_meshes();
    window.set_light(Light::StickToCamera);
    let state = AppState::new(&mut window);
    window.render_loop(state)
}
