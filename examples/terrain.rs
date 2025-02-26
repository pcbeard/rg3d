//! Example - Terrain.

extern crate rg3d;

pub mod shared;

use crate::shared::create_camera;
use rg3d::{
    core::{
        algebra::{UnitQuaternion, Vector2, Vector3},
        color::Color,
        pool::Handle,
        rand::Rng,
    },
    engine::{framework::prelude::*, resource_manager::ResourceManager},
    event::{ElementState, VirtualKeyCode, WindowEvent},
    event_loop::ControlFlow,
    gui::{
        message::{MessageDirection, TextMessage},
        text::TextBuilder,
        widget::WidgetBuilder,
    },
    rand::thread_rng,
    scene::{
        base::BaseBuilder,
        light::{point::PointLightBuilder, BaseLightBuilder},
        node::Node,
        terrain::{Brush, BrushMode, BrushShape, LayerDefinition, TerrainBuilder},
        transform::TransformBuilder,
        Scene,
    },
};

struct SceneLoader {
    scene: Scene,
    model_handle: Handle<Node>,
}

impl SceneLoader {
    async fn load_with(resource_manager: ResourceManager) -> Self {
        let mut scene = Scene::new();

        // Set ambient light.
        scene.ambient_lighting_color = Color::opaque(200, 200, 200);

        // Camera is our eyes in the world - you won't see anything without it.
        let model_handle = create_camera(
            resource_manager.clone(),
            Vector3::new(32.0, 6.0, 32.0),
            &mut scene.graph,
        )
        .await;

        // Add terrain.
        let terrain = TerrainBuilder::new(BaseBuilder::new())
            .with_layers(vec![
                LayerDefinition {
                    diffuse_texture: Some(
                        resource_manager.request_texture("examples/data/Grass_DiffuseColor.jpg"),
                    ),
                    normal_texture: Some(
                        resource_manager.request_texture("examples/data/Grass_Normal.jpg"),
                    ),
                    specular_texture: None,
                    roughness_texture: None,
                    height_texture: None,
                    tile_factor: Vector2::new(10.0, 10.0),
                },
                LayerDefinition {
                    diffuse_texture: Some(
                        resource_manager.request_texture("examples/data/Rock_DiffuseColor.jpg"),
                    ),
                    normal_texture: Some(
                        resource_manager.request_texture("examples/data/Rock_Normal.jpg"),
                    ),
                    specular_texture: None,
                    roughness_texture: None,
                    height_texture: None,
                    tile_factor: Vector2::new(10.0, 10.0),
                },
            ])
            .build(&mut scene.graph);

        let terrain = scene.graph[terrain].as_terrain_mut();

        for _ in 0..60 {
            let x = thread_rng().gen_range(4.0..60.00);
            let z = thread_rng().gen_range(4.0..60.00);
            let radius = thread_rng().gen_range(2.0..4.0);
            let height = thread_rng().gen_range(1.0..3.0);

            // Draw something on the terrain.

            // Pull terrain.
            terrain.draw(&Brush {
                center: Vector3::new(x, 0.0, z),
                shape: BrushShape::Circle { radius },
                mode: BrushMode::ModifyHeightMap { amount: height },
            });

            // Draw rock texture on top.
            terrain.draw(&Brush {
                center: Vector3::new(x, 0.0, z),
                shape: BrushShape::Circle { radius },
                mode: BrushMode::DrawOnMask {
                    layer: 1,
                    alpha: 1.0,
                },
            });
        }

        // Add some light.
        PointLightBuilder::new(BaseLightBuilder::new(
            BaseBuilder::new().with_local_transform(
                TransformBuilder::new()
                    .with_local_position(Vector3::new(0.0, 12.0, 0.0))
                    .build(),
            ),
        ))
        .with_radius(20.0)
        .build(&mut scene.graph);

        Self {
            scene,
            model_handle,
        }
    }
}

struct InputController {
    rotate_left: bool,
    rotate_right: bool,
}

struct Game {
    debug_text: Handle<UiNode>,
    input_controller: InputController,
    model_angle: f32,
    scene: Handle<Scene>,
    model_handle: Handle<Node>,
}

impl GameState for Game {
    fn init(engine: &mut GameEngine) -> Self
    where
        Self: Sized,
    {
        // Create test scene.
        let loader = rg3d::core::futures::executor::block_on(SceneLoader::load_with(
            engine.resource_manager.clone(),
        ));

        Self {
            debug_text: TextBuilder::new(WidgetBuilder::new())
                .build(&mut engine.user_interface.build_ctx()),
            input_controller: InputController {
                rotate_left: false,
                rotate_right: false,
            },
            scene: engine.scenes.add(loader.scene),
            model_angle: 180.0f32.to_radians(),
            model_handle: loader.model_handle,
        }
    }

    fn on_tick(&mut self, engine: &mut GameEngine, _dt: f32, _: &mut ControlFlow) {
        // Use stored scene handle to borrow a mutable reference of scene in
        // engine.
        let scene = &mut engine.scenes[self.scene];

        // Rotate model according to input controller state.
        if self.input_controller.rotate_left {
            self.model_angle -= 5.0f32.to_radians();
        } else if self.input_controller.rotate_right {
            self.model_angle += 5.0f32.to_radians();
        }

        scene.graph[self.model_handle]
            .local_transform_mut()
            .set_rotation(UnitQuaternion::from_axis_angle(
                &Vector3::y_axis(),
                self.model_angle,
            ));

        engine.user_interface.send_message(TextMessage::text(
            self.debug_text,
            MessageDirection::ToWidget,
            format!(
                "Example - Terrain\nUse [A][D] keys to rotate camera.\nFPS: {}",
                engine.renderer.get_statistics().frames_per_second
            ),
        ));
    }

    fn on_window_event(&mut self, _engine: &mut GameEngine, event: WindowEvent) {
        if let WindowEvent::KeyboardInput { input, .. } = event {
            // Handle key input events via `WindowEvent`, not via `DeviceEvent` (#32)
            if let Some(key_code) = input.virtual_keycode {
                match key_code {
                    VirtualKeyCode::A => {
                        self.input_controller.rotate_left = input.state == ElementState::Pressed
                    }
                    VirtualKeyCode::D => {
                        self.input_controller.rotate_right = input.state == ElementState::Pressed
                    }
                    _ => (),
                }
            }
        }
    }
}

fn main() {
    Framework::<Game>::new()
        .unwrap()
        .title("Example - Terrain")
        .run();
}
