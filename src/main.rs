use bevy::{math::vec2, prelude::*};
use bevy_prototype_lyon::prelude::*;
use bevy_svg::prelude::*;

const WIDTH: f32 = 500.;
const HEIGHT: f32 = 500.;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Ray casting test".to_string(),
            width: WIDTH,
            height: HEIGHT,
            present_mode: bevy::window::PresentMode::Fifo,
            ..default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(ShapePlugin)
        .add_plugin(bevy_svg::prelude::SvgPlugin)
        .add_startup_system(setup_system)
        .run();
}

// ! Work with only "line" element, and only if the first 4 attributes are x1,y1,x2,y2 !
// TODO: Make it so that it actually work.
pub fn get_points(file: &str) -> Vec<Vec<Vec2>> {
    use quick_xml::events::Event;
    use quick_xml::Reader;
    use std::fs;

    let file = fs::read_to_string(file).unwrap();
    let mut reader = Reader::from_str(&file);
    reader.trim_text(true);
    let mut points = Vec::<Vec<Vec2>>::new();
    loop {
        match reader.read_event_unbuffered() {
            Ok(Event::Empty(ref e)) => {
                // Assigns coordinates of points to vector in [[x1,y1],[x2,y2]] format
                let values = e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>();

                points.push(vec![
                    vec2(
                        String::from_utf8(values[0].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap(),
                        String::from_utf8(values[1].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap(),
                    ),
                    vec2(
                        String::from_utf8(values[2].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap(),
                        String::from_utf8(values[3].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap(),
                    ),
                ]);
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    points
}

// Create simple scene with line(s) and source.
fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let points = get_points("assets/image.svg");

    dbg!(points);

    let svg = asset_server.load("image.svg");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(Svg2dBundle {
        svg,
        origin: Origin::Center,
        ..Default::default()
    });
}
