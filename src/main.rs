// use std::{ops::Deref, ptr::Pointee};

use bevy::{prelude::*};
use petgraph::graph::UnGraph;
use uuid::Uuid;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter; // 0.17.1

use std::collections::HashMap;

use bevy_mod_bounding::{sphere::BSphere, debug, *};
use bevy_inspector_egui::WorldInspectorPlugin;


#[cfg(target_arch = "wasm32")]
use bevy_webgl2::{*};

struct Icon {
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

fn change_tool(mut query : Query<(&mut Visible, &mut Handle<StandardMaterial>, &mut Mesh), With<Icon>>, handle_map : ResMut<HandleMaterialMap>, tool_history: ResMut<ToolHistory>    ) {
    if tool_history.is_changed() {

        info!("The tool history has been changed.");

        if let Ok((mut visible,  mut handle, mut mesh)) = query.single_mut() {
            info!("Got the pbr object!");
            if let Some(current_material) = handle_map.tools.get(&tool_history.current_tool) {
                info!("making the sprite bundle visible and changing the color material (hopefully)");
                
                visible.is_transparent = false;
                visible.is_visible = true;
                *handle = current_material.to_owned();
                // Todo: change the other properties of the tools here... For instance, make the empty tool smaller. Probably I'll want to change the HandleMaterialMap to potentially have more properties to edit. I could also store a custom pbr bundle in each of the values of the map instead of the standard material. This would add complete customizability
            }
            
        }
    }
    // info!("The change_tool system has been triggered.");
    


}

fn adjust_cursor_position(window : &Res<Windows>) -> Option<(f32, f32)> {
    let window = window.get_primary().unwrap();

    

    let mut adjust_x : f32 = window.width()/2.0;
    let adjust_y = window.height()/2.0;
    

    
    if let Some(position) = window.cursor_position() {
        let mut x = position.x;
        let mut y = position.y;
        x = x - adjust_x;
        y = y - adjust_y;
        return Some((x,y));
    }
    return None
}

//bevy::math::f32::Vec3
fn change_cursor_position(windows: Res<Windows>,  mut query : Query<(&Icon, &mut Transform)>) {
    for (_potential_node, mut transform) in query.iter_mut() {
            // If the node is already existing on the screen somewhere, we should transform it to the position of the mouse! Instead of iterating through... There should only be one potential node on the screen at once.
    
        
                if let Some((x,y)) = adjust_cursor_position(&windows){
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
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(EguiPlugin)
        .add_plugin(BoundingVolumePlugin::<sphere::BSphere>::default())
        // .add_plugin(BoundingVolumePlugin::<obb::Obb>::default())
        .insert_resource(ToolHistory{
            current_tool: Tools::Empty,
            last_tool: None,
        })
        .insert_resource(LastPlacedEntity(None))
        .add_startup_system(setup.system())
        .add_system(change_cursor_position.system())
        .add_system(tool_menu.system())
        .add_system(change_tool.system())
        .add_system(check_what_is_clicked.system());
    
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    
    app.run();
}
/// This is a tag indicating the entities within the environment that have been placed on the grid.
#[derive(Debug)]
struct Placed{
    x : f32,
    y: f32,
    z: f32
}

struct LastPlacedEntity(Option<Entity>);

fn place_icon(current_tool : Tools , handle_map : ResMut<HandleMaterialMap>, mut commands : Commands, window : Res<Windows>,  mut meshes: ResMut<Assets<Mesh>>) {
    if let Some(material) = handle_map.tools.get(&current_tool.clone()){
        
                        if let Some((x,y)) = adjust_cursor_position(&window) {

                            info!("Should be placing {:?} at {} {}", current_tool, x, y);
                            commands.spawn_bundle(
                                PbrBundle{
                                    visible : Visible {
                                        is_visible: true,
                                        is_transparent: false,
                                    },
                                    mesh: meshes.add(
                                        Mesh::from(shape::Cube{
                                            size: handle_map.length
                                        })
                                    ),
                                    material : material.clone().to_owned(),
                                    transform: Transform::from_xyz(x.clone(), y.clone(), 0.0),
                                    ..Default::default()
                                }
                            )
                            .insert(Placed{x,y, z: 0.0}
                            )
                            .insert(Bounded::<sphere::BSphere>::default())
                            .insert(debug::DebugBounds);           
                        }



    }
}

fn check_what_is_clicked(egui_context: ResMut<EguiContext>, buttons: Res<Input<MouseButton>>, query : Query<(Entity, &Placed, &BSphere)>, window : Res<Windows>, commands : Commands, handle_map : ResMut<HandleMaterialMap>, tool_history: ResMut<ToolHistory>  , meshes: ResMut<Assets<Mesh>>, mut last_entity : ResMut<LastPlacedEntity>) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed
        // check what was clicked by having a query that looks up everything with a position
        let primary_window = window.get_primary().unwrap();
        if let Some(position) = primary_window.cursor_position() {
            if egui_context.ctx().is_pointer_over_area() {
                info!("Also clicked on a egui window, shouldn't try placing an icon.");
            }
            else {
                let current_tool = tool_history.current_tool.clone();

                    match current_tool.clone() {
                        Tools::Empty => {
                            for (entity, placed, bounded) in query.iter() {

                                if let Some(cursor) = adjust_cursor_position(&window){

                                    // info!("mouse info: {:?}\nbounded info: {:?}\nplaced info: {:?}", cursor, bounded, placed);

                                    let mesh_radius = bounded.mesh_space_radius();

                                    let (cursor_x, cursor_y) = cursor;
                                    let mouse_position = Vec3::new(cursor_x, cursor_y,0.0);
                                    let placed_position = Vec3::new(placed.x, placed.y, 0.0);
                                    
                                    if mouse_position.distance(placed_position) < *mesh_radius {
                                        info!("I should select the icon here. It is represented by the entity {:?}", entity);
                                        *last_entity = LastPlacedEntity(Some(entity.clone()));
                                    }
                                }

                                 
                            }
                        },
                        Tools::Node => {
                            info!("The window was clicked at: (x,y) : ({},{})", position.x, position.y);
                            place_icon(current_tool,handle_map,commands,window, meshes);
                        },
                        Tools::Edge => {                

                        },
                    }

    
        }
    }


}
}

fn setup(mut commands : Commands, mut materials : ResMut<Assets<StandardMaterial>>,     mut meshes: ResMut<Assets<Mesh>>,) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d() );
    // commands.insert_resource(Graph(UnGraph::new().add_node(Node::new())))
let mut handle_map : HandleMaterialMap = HandleMaterialMap {
    tools : HashMap::new(),
    length : 30.0,
    height: 17.0
};

commands.spawn_bundle(PbrBundle{
    // sprite: Sprite::new(Vec2::new(handle_map.length, handle_map.height)),
    mesh: meshes.add(Mesh::from(shape::Cube{
        size: handle_map.length,
    })),
    visible: Visible {
        is_visible: true,
        is_transparent: false,
    },
    transform: Transform{
        translation: Vec3::new(0.0, 0.0, 0.0),
        ..Default::default()
    },
    ..Default::default()
}).insert(Icon{
    
    current_tool : Tools::Empty
});


// Oh, right, pbr requires light :shocked_pichachu:
commands.spawn_bundle(LightBundle {
    transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
    light: Light{
        intensity: 1.0,
        ..Default::default()
    },
    ..Default::default()
});

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
                base_color: Color::Rgba{ red: 0.0, green: 1.0, blue: 0.0, alpha: 0.0 },
                roughness: 0.7,
                metallic: 0.7,
                unlit : false,
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
        
        for variant in Tools::iter() {
            match variant {
                Tools::Empty => {
                    let node_button = ui.add(
                        egui::Button::new("Selector Tool")
                    
                    );
                    if node_button.clicked() {
                            let last_tool = tool_history.current_tool.clone();
                            tool_history.current_tool = Tools::Empty;  
                            tool_history.last_tool = Some(last_tool);
                            bevy::log::info!("Selected the selector tool!");
                        
                    };
                },
                Tools::Node => {
                    let node_button = ui.add(
                        egui::Button::new("Node Tool")
                    
                    );
                    if node_button.clicked() {
                            let last_tool = tool_history.current_tool.clone();
                            tool_history.current_tool = Tools::Node;
                            tool_history.last_tool = Some(last_tool);
                            bevy::log::info!("Selected the node tool!");
                        
                    };
                },
                Tools::Edge => {
                    
        let edge_button = ui.add(
            egui::Button::new("Edge Tool")
        
        );
        if edge_button.clicked() {
            let last_tool = tool_history.current_tool.clone();
            tool_history.current_tool = Tools::Edge;
            tool_history.last_tool = Some(last_tool);
                bevy::log::info!("Selected the edge tool!");
            
        };
                },
            }
        }
        
        


    });
}