// use std::{ops::Deref, ptr::Pointee};

use bevy::{prelude::*};
use petgraph::graph::UnGraph;
use uuid::Uuid;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

use std::collections::HashMap;

use bevy_mod_bounding::{obb, debug, *};


#[cfg(target_arch = "wasm32")]
use bevy_webgl2::{*};

struct Icon {
    position: Position,
    label : String,
    current_tool : Tools
}

struct Position {
    x : f32,
    y : f32,
    z : f32
}

struct HandleMaterialMap {
    tools: HashMap<Tools,Handle<StandardMaterial>>,
    length : f32,
    height: f32
}

struct Node {
    identity : uuid::Uuid,
    label : String,
}

impl Node {
    fn new() -> Self {
    Node { identity: Uuid::new_v4(), label: String::from("My First Node!") }
    }
}

fn change_tool(mut query : Query<(&mut Visible, &mut Handle<StandardMaterial>), With<Icon>>, handle_map : ResMut<HandleMaterialMap>, tool_history: ResMut<ToolHistory>    ) {
    if tool_history.is_changed() {

        info!("The tool history has been changed.");

        if let Ok((mut visible,  mut handle )) = query.single_mut() {
            info!("Got the pbr object!");
            if let Some(current_material) = handle_map.tools.get(&tool_history.current_tool) {
                info!("making the sprite bundle visible and changing the color material (hopefully)");
                
                visible.is_transparent = false;
                visible.is_visible = true;
                *handle = current_material.to_owned();
            }
            
        }
    }
    // info!("The change_tool system has been triggered.");
    


}

//bevy::math::f32::Vec3
fn place_node(windows: Res<Windows>,  mut query : Query<(&Icon, &mut Transform)>) {

    let window = windows.get_primary().unwrap();
    let adjust_x = window.width()/2.0;
    let adjust_y = window.height()/2.0;
    
    for (_potential_node, mut transform) in query.iter_mut() {
            // If the node is already existing on the screen somewhere, we should transform it to the position of the mouse! Instead of iterating through... There should only be one potential node on the screen at once.
    
            if let Some(position) = window.cursor_position() {


                let x = position.x.to_owned() - adjust_x;
                let y = position.y.to_owned() - adjust_y;
                // cursor is inside the window, position given

                
                // info!("The cursor is at the postion: x: {}, y: {}",x,y);
                // Update the position of the sprite
                transform.translation.x = x;
                transform.translation.y = y;
            } 
    }


    
}

struct Graph(UnGraph<Node, ()>);

#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
enum Tools {
Empty,
Node,
Edge
}

struct ToolHistory{
    current_tool : Tools,
    last_tool : Option<Tools>
}


pub fn main() {

    let mut app = App::build();
    
    app
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(EguiPlugin)
        .add_plugin(BoundingVolumePlugin::<obb::Obb>::default())
        .insert_resource(ToolHistory{
            current_tool: Tools::Empty,
            last_tool: None,
        })
        .add_startup_system(setup.system().label("first"))
        .add_system_set(
            SystemSet::new()
                .after("first")
                .with_system(place_node.system()))

        .add_system(tool_menu.system())
        .add_system(change_tool.system());
    
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    
    app.run();
}
/// This is a tag indicating the entities within the environment that have been placed on the grid.
struct Placed;

fn check_what_is_clicked(egui_context: ResMut<EguiContext>, buttons: Res<Input<MouseButton>>, query : Query<&Placed>, window : Res<Windows>) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed
        // check what was clicked by having a query that looks up everything with a position


        let window = window.get_primary().unwrap();
        let position = window.cursor_position().unwrap();
        
        info!("The window was clicked at: (x,y) : ({},{})", position.x, position.y);
    }


}

fn setup(mut commands : Commands, mut materials : ResMut<Assets<StandardMaterial>>,     mut meshes: ResMut<Assets<Mesh>>,) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d() );
    // commands.insert_resource(Graph(UnGraph::new().add_node(Node::new())))
let mut handle_map : HandleMaterialMap = HandleMaterialMap {
    tools : HashMap::new(),
    length : 10.0,
    height: 10.0
};

commands.spawn_bundle(PbrBundle{
    // sprite: Sprite::new(Vec2::new(handle_map.length, handle_map.height)),
    mesh: meshes.add(Mesh::from(shape::Cube{
        size: handle_map.length,
    })),
    visible: Visible {
        is_visible: false,
        is_transparent: false,
    },
    transform: Transform{
        translation: Vec3::new(0.0, 0.0, 0.0),
        ..Default::default()
    },
    ..Default::default()
}).insert(Icon{
    position: Position{x :0.0,y:0.0,z:0.0},
    label: String::from("Cursor"),
    current_tool : Tools::Empty
})
.insert(Bounded::<obb::Obb>::default())
.insert(debug::DebugBounds);

// for each of the potentially selected tools, let's insert a resource to represent that tool. Using this construction will guarantee that the system will not compile unless there is an allocated resource for each of the tools.
for variant in Tools::iter() {
    match variant {
        Tools::Edge => {
            let handle = materials.add(StandardMaterial{
                base_color: Color::Rgba{ red: 1.0, green: 0.0, blue: 0.0, alpha: 0.5 },
                roughness: 0.7,
                metallic: 0.7,
                ..Default::default()
            });
            handle_map.tools.insert(Tools::Edge, handle.clone());
            
        }
        Tools::Empty => {
            let handle = materials.add(StandardMaterial{
                base_color: Color::Rgba{ red: 0.0, green: 1.0, blue: 0.0, alpha: 0.5 },
                roughness: 0.7,
                metallic: 0.7,
                ..Default::default()
            });
            handle_map.tools.insert(Tools::Empty, handle.clone());
            
        }
        Tools::Node => {
            let handle = materials.add(StandardMaterial{
                base_color: Color::Rgba{ red: 0.0, green: 0.0, blue: 1.0, alpha: 0.5 },
                roughness: 0.7,
                metallic: 0.7,
                unlit: false,
                ..Default::default()
                
            });
            handle_map.tools.insert(Tools::Node, handle.clone());
            
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
fn tool_menu(egui_context: ResMut<EguiContext>, mut tool_history : ResMut<ToolHistory>) {
    egui::Window::new("Toolbox").show(egui_context.ctx(), |ui| {
        let node_button = ui.add(
            egui::Button::new("Node Tool")
        
        );
        if node_button.clicked() {
                let last_tool = tool_history.current_tool.clone();
                tool_history.current_tool = Tools::Node;
                tool_history.last_tool = Some(last_tool);
                bevy::log::info!("Selected the node tool!");
            
        };
        let edge_button = ui.add(
            egui::Button::new("Edge Tool")
        
        );
        if edge_button.clicked() {
            let last_tool = tool_history.current_tool.clone();
            tool_history.current_tool = Tools::Edge;
            tool_history.last_tool = Some(last_tool);
                bevy::log::info!("Selected the edge tool!");
            
        };

    });
}