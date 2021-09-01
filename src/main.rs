// use std::{ops::Deref, ptr::Pointee};

use bevy::{prelude::*};
use petgraph::graph::UnGraph;
use uuid::Uuid;

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

struct NodeMaterial {
    placed_color : Handle<ColorMaterial>,
    potential_color : Handle<ColorMaterial>,
    length: f32,
    height: f32,
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

//bevy::math::f32::Vec3
fn place_node(windows: Res<Windows>, mut commands : Commands, materials : Res<NodeMaterial>, mut query : Query<(&PotentialNode, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let adjust_x = window.width()/2.0;
    let adjust_y = window.height()/2.0;

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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[derive(SystemLabel)]
enum MySystem {
ResourceInitialization
}


pub fn main() {
    //   When building for WASM, print panics to the browser console
    //   #[cfg(target_arch = "wasm32")]
    //   console_error_panic_hook::set_once();

    let mut app = App::build();
    
    app
        .add_plugins(bevy::DefaultPlugins)
        .add_startup_system(setup.system().label("first"))
        .add_system_set(
            SystemSet::new()
                .after("first")
                .with_system(place_node.system())
    );
    
    // when building for Web, use WebGL2 rendering
    #[cfg(target_arch = "wasm32")]
    app.add_plugin(bevy_webgl2::WebGL2Plugin);
    
    

    app.run();
}

fn setup(mut commands : Commands, mut materials : ResMut<Assets<ColorMaterial>>) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d() );
    // commands.insert_resource(Graph(UnGraph::new().add_node(Node::new())))
commands.insert_resource(NodeMaterial{ placed_color: materials.add(ColorMaterial::color(Color::BLACK)), potential_color: materials.add(ColorMaterial::color(Color::GREEN)), length: 50.0, height: 50.0})

}