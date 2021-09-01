// use std::{ops::Deref, ptr::Pointee};

use bevy::{prelude::*};
use petgraph::graph::UnGraph;
use uuid::Uuid;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1
use std::collections::HashMap;


#[cfg(target_arch = "wasm32")]
use bevy_webgl2::{*};

struct PotentialNode {
    position: Position,
    label : String,
}

struct Position {
    x : f32,
    y : f32,
    z : f32
}

struct HandleMaterialMap {
    tools: HashMap<SelectedTool,Handle<ColorMaterial>>,
    length : f32,
    height: f32
}


struct Node {
    identity : uuid::Uuid,
    label : String
}

impl Node {
    fn new() -> Self {
    Node { identity: Uuid::new_v4(), label: String::from("My First Node!") }
    }
}

fn spawn_node(mut commands : Commands, materials : Res<NodeMaterial>) {
    
    // let local_materials = materials.deref().clone();
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite::new(Vec2::new(materials.length, materials.height)),
            material: materials.placed_color.clone(),
            ..Default::default()
        })
        .insert(Node::new());
}

fn change_tool(selected_tool : Res<SelectedTool>) {
    
}

//bevy::math::f32::Vec3
fn place_node(windows: Res<Windows>, mut commands : Commands, materials : Res<NodeMaterial>, mut query : Query<(&PotentialNode, &mut Transform)>, selected_tool : Res<SelectedTool>) {
    let window = windows.get_primary().unwrap();
    let adjust_x = window.width()/2.0;
    let adjust_y = window.height()/2.0;
    match selected_tool {
        SelectedTool::Empty => todo!(),
        SelectedTool::Node => todo!(),
        SelectedTool::Edge => todo!(),
    }

    let mut count = 0;
    for (_potential_node, mut transform) in query.iter_mut() {
        count = count + 1;
            // If the node is already existing on the screen somewhere, we should transform it to the position of the mouse! Instead of iterating through... There should only be one potential node on the screen at once.
    
            if let Some(position) = window.cursor_position() {
                let x = position.x.to_owned() - adjust_x;
                let y = position.y.to_owned() - adjust_y;
                // cursor is inside the window, position given
    
                // Update the position of the sprite
                transform.translation.x = x;
                transform.translation.y = y;
            } 
    }

    if count == 0 {
            // The potential node has not been added to the resources available to bevy! We need to add the potential node to the resource repository
    
            if let Some(position) = window.cursor_position() {
                let x = position.x.to_owned() - adjust_x;
                let y = position.y.to_owned() - adjust_y;
                // cursor is inside the window, position given
                commands.spawn_bundle(SpriteBundle{
                    sprite: Sprite::new(Vec2::new(materials.length, materials.height)),
                    material: materials.potential_color.clone(),
                    transform: Transform{
                        translation: Vec3::new(x, y, 0.0),
                        ..Default::default()
                    },
                    ..Default::default()
                }).insert(PotentialNode{
                    position: Position{x,y,z:0.0},
                    label: String::from("Potential Node"),
                });
    
            } else {
                // cursor is not inside the window and so we will not yet spawn the resource... Because there's no sensible place to put it :'(
            }
    }

    

    

    
}

struct Graph(UnGraph<Node, ()>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
enum SelectedTool {
Empty,
Node,
Edge
}


pub fn main() {
    //   When building for WASM, print panics to the browser console
    //   #[cfg(target_arch = "wasm32")]
    //   console_error_panic_hook::set_once();

    let mut app = App::build();
    
    app
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(EguiPlugin)
        .insert_resource(SelectedTool::Empty)
        .add_startup_system(setup.system().label("first"))
        .add_system_set(
            SystemSet::new()
                .after("first")
                .with_system(place_node.system()))

        .add_system(tool_menu.system());
    
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    
    

    app.run();
}

fn setup(mut commands : Commands, mut materials : ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d() );
    // commands.insert_resource(Graph(UnGraph::new().add_node(Node::new())))
let mut handle_map : HandleMaterialMap = HandleMaterialMap {
    tools : HashMap<SelectedTool,Handle<ColorMaterial>>::new(),
    length : 10.0,
    height: 10.0
};

// for each of the potentially selected tools, let's insert a resource to represent that tool. Using this construction will guarantee that the system will not compile unless there is an allocated resource for each of the tools.
for variant in SelectedTool.iter() {
    match variant {
        SelectedTool::Edge => {
            let handle = materials.add(ColorMaterial::color(Color::BLACK));
            handle_map.tools.insert(SelectedTool::Edge, handle);
        }
        SelectedTool::Empty => {
            let handle = materials.add(ColorMaterial::color(Color::Hsla(0.0,0.0,0.0,0.0)));
            handle_map.tools.insert(SelectedTool::Edge, handle);
        }
        SelectedTool::Node => {
            let handle = materials.add(ColorMaterial::color(Color::GREEN));
            handle_map.tools.insert(SelectedTool::Edge, handle);
        }
        
    }
}

commands.insert_resource(handle_map);

}


// Egui stuff will go below here :]

pub fn update_ui_scale_factor(
    keyboard_input: Res<Input<KeyCode>>,
    mut toggle_scale_factor: Local<Option<bool>>,
    mut egui_settings: ResMut<EguiSettings>,
    windows: Res<Windows>,
) {
    if keyboard_input.just_pressed(KeyCode::Slash) || toggle_scale_factor.is_none() {
        *toggle_scale_factor = Some(!toggle_scale_factor.unwrap_or(true));

        if let Some(window) = windows.get_primary() {
            let scale_factor = if toggle_scale_factor.unwrap() {
                1.0
            } else {
                1.0 / window.scale_factor()
            };
            egui_settings.scale_factor = scale_factor;
        }
    }
}

// Note the usage of `ResMut`. Even though `ctx` method doesn't require
// mutability, accessing the context from different threads will result
// into panic if you don't enable `egui/multi_threaded` feature.
fn tool_menu(egui_context: ResMut<EguiContext>, mut selected_tool : ResMut<SelectedTool>) {
    egui::Window::new("Toolbox").show(egui_context.ctx(), |ui| {
        let node_button = ui.add(
            egui::Button::new("Node Tool")
        
        );
        if node_button.clicked() {
                *selected_tool = SelectedTool::Node;
                bevy::log::info!("Selected the node tool!");
            
        };
        let edge_button = ui.add(
            egui::Button::new("Edge Tool")
        
        );
        if edge_button.clicked() {
                *selected_tool = SelectedTool::Edge;
                bevy::log::info!("Selected the edge tool!");
            
        };

    });
}