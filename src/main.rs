// use std::{ops::Deref, ptr::Pointee};

use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin, EguiSettings};
use either::Either;
use petgraph::stable_graph::StableGraph;
use strum::IntoEnumIterator; // 0.17.1
use strum_macros::EnumIter;
use uuid::Uuid; // 0.17.1
use Either::{Left, Right};

use std::collections::HashMap;

use bevy_inspector_egui::WorldInspectorPlugin;
use bevy_mod_bounding::{debug, sphere::BSphere, *};
use petgraph::stable_graph::NodeIndex;

#[cfg(target_arch = "wasm32")]
use bevy_webgl2::*;

struct Cursor {
    current_tool: Tools,
}

#[derive(Debug, Clone)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

struct HandleMaterialMap {
    tools: HashMap<Tools, Handle<StandardMaterial>>,
    length: f32,
    height: f32,
}

#[derive(Clone)]
struct Node {
    label: String,
    position: Position,
    identity: Option<NodeIndex>
}
enum GraphInteraction {
    AddedNode(Node),
    AddedEdge(Edge),
    RemovedNode(Node),
    RemovedEdge(Edge),
}

struct GraphInteractionHistory(Vec<GraphInteraction>);


#[derive(Clone)]
struct Edge {
    node_a : NodeIndex,
    node_b : NodeIndex
}


use std::sync::Arc;

struct Graph(StableGraph<Node, Edge>);



#[derive(Debug, Clone, PartialEq, Eq, Hash, EnumIter)]
enum Tools {
    Selector,
    Node,
    Edge,
}

type EntityType = Tools;

struct ToolHistory {
    current_tool: Tools,
    last_tool: Option<Tools>,
}

/// This struct will allow the application to implement undo capabilities at some point in the future. Presently, it is used for determining if tools requiring more than one click are complete in their task. For instance, if a `Tools::Edge` interacts with a node, we want to know if this is the second time this has taken place -- indicating that an edge should connect the two nodes.
struct InteractionHistory {
    /// The first optional tuple represents the entity (and its corresponding type) that has been clicked by the Tool.
    history: Vec<Either<(Option<(Entity, EntityType)>, Tools), ActionTaken>>,
}
/// This fella represents the case in which case an interaction has been parsed and enacted. This will make it so that the next interaction doesn't read past interactions that have already been placed. For instance, in the case that the edge tool is selected and three different nodes A, B and then C are clicked. Without adding the ActionTaken to the interaction history, an edge would be added between A and B and then also an edge between B and C. This is not the desired behavior. When the enact_interaction system is triggered, it will add this struct to the `InteractionHistory`.
struct ActionTaken;

/// This is a tag indicating the entities within the environment that have been placed on the grid.
#[derive(Debug)]
struct Placed {
    position: Position,
    entity_type: Tools,
}

pub fn main() {
    let mut app = App::build();

    app.add_plugins(bevy::DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(EguiPlugin)
        .add_plugin(BoundingVolumePlugin::<sphere::BSphere>::default())
        // .add_plugin(BoundingVolumePlugin::<obb::Obb>::default())
        .insert_resource(ToolHistory {
            current_tool: Tools::Selector,
            last_tool: None,
        })
        .insert_resource(InteractionHistory {
            history: Vec::new(),
        })
        .insert_resource(GraphInteractionHistory(Vec::new()))
        // .insert_resource(LastClickedEntity(None))
        .add_startup_system(setup.system())
        .add_system(visualize_graph.system())
        .add_system(change_cursor_position.system())
        .insert_resource(Graph(
            StableGraph::new(),
            
        ))
        .add_system(tool_menu.system())
        .add_system(change_tool.system())
        .add_system(check_what_is_clicked.system())
        .add_system(enact_interaction.system());

    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);

    app.run();
}

fn change_tool(
    mut query: Query<(&mut Visible, &mut Handle<StandardMaterial>, &mut Mesh), With<Cursor>>,
    handle_map: ResMut<HandleMaterialMap>,
    tool_history: ResMut<ToolHistory>,
) {
    if tool_history.is_changed() {
        info!("The tool history has been changed.");

        if let Ok((mut visible, mut handle, mut mesh)) = query.single_mut() {
            info!("Got the pbr object!");
            if let Some(current_material) = handle_map.tools.get(&tool_history.current_tool) {
                info!(
                    "making the sprite bundle visible and changing the color material (hopefully)"
                );

                visible.is_transparent = false;
                visible.is_visible = true;
                *handle = current_material.to_owned();
                // Todo: change the other properties of the tools here... For instance, make the Selector tool smaller. Probably I'll want to change the HandleMaterialMap to potentially have more properties to edit. I could also store a custom pbr bundle in each of the values of the map instead of the standard material. This would add complete customizability
            }
        }
    }
    // info!("The change_tool system has been triggered.");
}

fn adjust_cursor_position(window: &ResMut<Windows>, optional_position : Option<Position>) -> Option<(f32, f32)> {
    let window = window.get_primary().unwrap();

    let mut adjust_x: f32 = window.width() / 2.0;
    let adjust_y = window.height() / 2.0;

    if let Some(position) = optional_position {
        let mut x = position.x;
        let mut y = position.y;
        x = x - adjust_x;
        y = y - adjust_y;
        return Some((x, y));
    }
    else if let Some(position) = window.cursor_position() {
        let mut x = position.x;
        let mut y = position.y;
        x = x - adjust_x;
        y = y - adjust_y;
        return Some((x, y));
    }
    return None;
}

//bevy::math::f32::Vec3
fn change_cursor_position(windows: ResMut<Windows>, mut query: Query<(&Cursor, &mut Transform)>) {
    for (_potential_node, mut transform) in query.iter_mut() {
        // If the node is already existing on the screen somewhere, we should transform it to the position of the mouse! Instead of iterating through... There should only be one potential node on the screen at once.

        if let Some((x, y)) = adjust_cursor_position(&windows, None) {
            // info!("The cursor is at the postion: x: {}, y: {}",x,y);
            // Update the position of the sprite
            transform.translation.x = x;
            transform.translation.y = y;
        }
    }
}

fn enact_interaction(
    mut interaction: ResMut<InteractionHistory>,
    handle_map: ResMut<HandleMaterialMap>,
    mut commands: Commands,
    window: ResMut<Windows>,
    node_query : Query<(Entity,&Node)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut graph : ResMut<Graph>,
    mut graph_interaction_history : ResMut<GraphInteractionHistory>
) {
    if interaction.is_changed() {

        if let Some(Right(ActionTaken)) = interaction.history.last() {
            info!("Did an action.");
        }


        // In the case that the last interaction was not actually an action taken. In which case we don't want to take any action.
        if let Some(Left((Some((entity, entity_type)), interacting_tool))) =
            interaction.history.last()
        {
            let mut entity_tool_interaction = (entity_type, interacting_tool);
            // The tool interacted with an entity... let's figure out what action to take... The main interaction will be placing an edge between two nodes if the last and second to last interactions were both acting on nodes with the edge

            let mut second_to_last_interaction_history: Option<
                &Either<(Option<(Entity, EntityType)>, Tools), ActionTaken>,
            > = None;

            if interaction.history.len() >= 2 {
                let second_to_last_index = interaction.history.len() - 2;
                second_to_last_interaction_history = interaction.history.get(second_to_last_index);
            }

            let mut second_to_last_entity_tool_interaction: Option<(&Entity, &EntityType, &Tools)> =
                None;

            if let Some(Left((Some((entity, entity_type)), interacting_tool))) =
                second_to_last_interaction_history
            {
                second_to_last_entity_tool_interaction =
                    Some((entity, entity_type, interacting_tool));
            }

            match entity_tool_interaction {
                (Tools::Selector, _) => {
                    // nothing to be done, there is no entity type associated with the selector tool
                }
                (Tools::Node, Tools::Selector) => {
                    info!("Bring up the node on the info window")
                }
                (Tools::Node, Tools::Node) => {
                    info!("The intent is probably to bring up an info panel of the node clicked")
                }
                (Tools::Node, Tools::Edge) => {
                    // This is the case in which an edge is being added to a node. We need to determine if the second to last interaction was the node tool interacting with a *different* node than the last node.
                    if let Some((last_entity, EntityType::Node, Tools::Edge)) =
                        second_to_last_entity_tool_interaction
                    {
                        if *entity != *last_entity {
                            info!("should add a node between the two entities here... Let's see if this even compiles.");
                            // I don't think that this actually triggers the interaction resource to be noted as changed since it is happening in this system itself (instead of in an external one... I still think it's important to register this though.)
                            let mut node_a : Option<Node> = None;
                            let mut node_b : Option<Node> = None;

                            for (check_entity, node) in node_query.iter() {
                                if check_entity == *entity {
                                    node_b = Some(node.clone());
                                }
                                if check_entity == *last_entity {
                                    node_a = Some(node.clone());
                                }
                            }

                            if node_b.is_some() && node_a.is_some() {
                                let node_index_a = node_a.unwrap().identity.unwrap();
                                let node_index_b = node_b.unwrap().identity.unwrap();
                                
                                let weight = Edge{
                                    node_a: node_index_a,
                                    node_b : node_index_b
                                };
                                graph.0.add_edge(node_index_a, node_index_b, weight.clone());
                                graph_interaction_history.0.push(GraphInteraction::AddedEdge(weight.clone()))
                            }
                            


                            interaction.history.push(Right(ActionTaken));
                        }
                    }
                }
                (Tools::Edge, Tools::Selector) => {
                    info!("bring up this edge in the info panel, highlight the connector that was clicked")
                },
                (Tools::Edge, Tools::Node) => 
                {
                    // This interaction doesn't make sense. Why would someone click a placed edge with the Node tool?
                },
                (Tools::Edge, Tools::Edge) => 
                {
                    info!("bring up the info panel containing the information about the edge")
                },
            }
        }
        if let Some(Left((None, interacting_tool))) = interaction.history.last() {
            // This is the case in which no component is selected but a tool is being used essentially on empty space
            match interacting_tool {
                Tools::Selector => {}
                Tools::Node => {

                    if let Some((x,y)) = adjust_cursor_position(&window, None) {

                    
                            let mut node = Node{
                                identity: None,
                                label: String::from(""),
                                position: Position { x, y, z: 0.0 }
                            };
                            let index = graph.0.add_node(node.clone());
        
                            if let Some(node_weight) = graph.0.node_weight_mut(index.clone()) {
                                node_weight.identity = Some(index);
                                info!("updated the node weight... This is ideal.")
                            }
                            
                            // node.identity = Some(index.clone());

                            let interaction = GraphInteraction::AddedNode(node.clone());

                            graph_interaction_history.0.push(interaction);
                        }
        
                            
                     

                   },
                Tools::Edge => {}
            }
        }
    }
}


fn visualize_graph(graph : ResMut<Graph>, mut graph_history : ResMut<GraphInteractionHistory>, handle_map : ResMut<HandleMaterialMap>, commands : Commands, meshes: ResMut<Assets<Mesh>>, window : ResMut<Windows>) {
    if graph.is_changed() || graph_history.is_changed() {
        info!("graph history changed");
        if let Some(last_interaction) = graph_history.0.last(){

            match last_interaction {
                GraphInteraction::AddedNode(node) => {
                    if let Some((x,y)) = adjust_cursor_position(&window, Some(node.position.clone())){
                    place_icon(Tools::Node,node.position.clone() ,handle_map, commands, meshes, window);   
                    }
                },
                GraphInteraction::AddedEdge(_) => todo!(),
                GraphInteraction::RemovedNode(_) => todo!(),
                GraphInteraction::RemovedEdge(_) => todo!(),
            }
            info!("should have changed SOMETHING about the graph");
        }
    }
}

fn place_icon(
    current_tool: Tools,
    position : Position,
    handle_map: ResMut<HandleMaterialMap>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    window : ResMut<Windows>,
) {

    if let Some(material) = handle_map.tools.get(&current_tool.clone()) {
        let x = position.x;
        let y = position.y;
        let z : f32 = 0.0;
        info!("Placing the {:?} at {:?}", current_tool, position);
            commands
                .spawn_bundle(PbrBundle {
                    visible: Visible {
                        is_visible: true,
                        is_transparent: false,
                    },
                    mesh: meshes.add(Mesh::from(shape::Cube {
                        size: handle_map.length,
                    })),
                    material: material.clone().to_owned(),
                    transform: Transform::from_xyz(x.clone(), y.clone(), z),
                    ..Default::default()
                })
                .insert(Placed {
                    position: position,
                    entity_type: current_tool.clone(),
                })
                .insert(Bounded::<sphere::BSphere>::default())
                .insert(debug::DebugBounds);
        
    }
}

fn check_what_is_clicked(
    egui_context: ResMut<EguiContext>,
    buttons: Res<Input<MouseButton>>,
    query: Query<(Entity, &Placed, &BSphere)>,
    window: ResMut<Windows>,
    commands: Commands,
    handle_map: ResMut<HandleMaterialMap>,
    tool_history: ResMut<ToolHistory>,
    meshes: ResMut<Assets<Mesh>>,

    mut interaction_history: ResMut<InteractionHistory>,
) {
    if buttons.just_pressed(MouseButton::Left) {
        // Left button was pressed
        // check what was clicked by having a query that looks up everything with a position
        let primary_window = window.get_primary().unwrap();
        if let Some(position) = primary_window.cursor_position() {
            if egui_context.ctx().is_pointer_over_area() {
                info!("Also clicked on a egui window, shouldn't try placing an icon.");
            } else {
                let current_tool = tool_history.current_tool.clone();

                for (entity, placed, bounded) in query.iter() {
                    if let Some(cursor) = adjust_cursor_position(&window, None) {
                        // info!("mouse info: {:?}\nbounded info: {:?}\nplaced info: {:?}", cursor, bounded, placed);

                        let mesh_radius = bounded.mesh_space_radius();

                        let (cursor_x, cursor_y) = cursor;
                        let mouse_position = Vec3::new(cursor_x, cursor_y, 0.0);
                        let placed_position = Vec3::new(placed.position.x, placed.position.y, 0.0);

                        // If we are inside the bound of the placed entity
                        if mouse_position.distance(placed_position) < *mesh_radius {
                            info!("I should select the icon here. It is represented by the entity {:?}", entity);

                            interaction_history.history.push(Left((
                                Some((entity.clone(), placed.entity_type.clone())),
                                current_tool.clone(),
                            )));
                            // we do not want to potentially register clicking two entities at the same time. If we accidentally click on two bounding boxes at the same time (where they overlap), then we need to break this loop
                            return;
                        }
                    }
                }
                // This means that no entity has been clicked, nor has the egui interface... So... if the tool is a `Tools::Node` we should place a node
                interaction_history
                    .history
                    .push(Left((None, current_tool.clone())));
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    // commands.insert_resource(Graph(UnGraph::new().add_node(Node::new())))
    let mut handle_map: HandleMaterialMap = HandleMaterialMap {
        tools: HashMap::new(),
        length: 30.0,
        height: 17.0,
    };

    commands
        .spawn_bundle(PbrBundle {
            // sprite: Sprite::new(Vec2::new(handle_map.length, handle_map.height)),
            mesh: meshes.add(Mesh::from(shape::Cube {
                size: handle_map.length,
            })),
            visible: Visible {
                is_visible: true,
                is_transparent: false,
            },
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Cursor {
            current_tool: Tools::Selector,
        });

    // Oh, right, pbr requires light :shocked_pichachu:
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        light: Light {
            intensity: 1.0,
            ..Default::default()
        },
        ..Default::default()
    });

    // for each of the potentially selected tools, let's insert a resource to represent that tool. Using this construction will guarantee that the system will not compile unless there is an allocated resource for each of the tools.
    for variant in Tools::iter() {
        match variant {
            Tools::Edge => {
                let handle = materials.add(StandardMaterial {
                    base_color: Color::Rgba {
                        red: 1.0,
                        green: 0.0,
                        blue: 0.0,
                        alpha: 0.5,
                    },
                    roughness: 0.7,
                    metallic: 0.7,
                    ..Default::default()
                });
                handle_map.tools.insert(Tools::Edge, handle.clone());
            }
            Tools::Selector => {
                let handle = materials.add(StandardMaterial {
                    base_color: Color::Rgba {
                        red: 0.0,
                        green: 1.0,
                        blue: 0.0,
                        alpha: 0.0,
                    },
                    roughness: 0.7,
                    metallic: 0.7,
                    unlit: false,
                    ..Default::default()
                });
                handle_map.tools.insert(Tools::Selector, handle.clone());
            }
            Tools::Node => {
                let handle = materials.add(StandardMaterial {
                    base_color: Color::Rgba {
                        red: 0.0,
                        green: 0.0,
                        blue: 1.0,
                        alpha: 0.5,
                    },
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
fn tool_menu(egui_context: ResMut<EguiContext>, mut tool_history: ResMut<ToolHistory>) {
    egui::Window::new("Toolbox").show(egui_context.ctx(), |ui| {
        for variant in Tools::iter() {
            match variant {
                Tools::Selector => {
                    let node_button = ui.add(egui::Button::new("Selector Tool"));
                    if node_button.clicked() {
                        let last_tool = tool_history.current_tool.clone();
                        tool_history.current_tool = Tools::Selector;
                        tool_history.last_tool = Some(last_tool);
                        bevy::log::info!("Selected the selector tool!");
                    };
                }
                Tools::Node => {
                    let node_button = ui.add(egui::Button::new("Node Tool"));
                    if node_button.clicked() {
                        let last_tool = tool_history.current_tool.clone();
                        tool_history.current_tool = Tools::Node;
                        tool_history.last_tool = Some(last_tool);
                        bevy::log::info!("Selected the node tool!");
                    };
                }
                Tools::Edge => {
                    let edge_button = ui.add(egui::Button::new("Edge Tool"));
                    if edge_button.clicked() {
                        let last_tool = tool_history.current_tool.clone();
                        tool_history.current_tool = Tools::Edge;
                        tool_history.last_tool = Some(last_tool);
                        bevy::log::info!("Selected the edge tool!");
                    };
                }
            }
        }
    });
}
