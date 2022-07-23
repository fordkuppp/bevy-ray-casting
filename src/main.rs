use bevy::{math::vec2, prelude::*};
use bevy_prototype_lyon::prelude::*;
use bevy_svg::prelude::*;
use iter_num_tools::lin_space;
use quick_xml::events::Event;
use quick_xml::Reader;
use std::fs;
use rayon::ThreadPoolBuilder;
use std::time::Instant;

const WIDTH: f32 = 500.;
const HEIGHT: f32 = 500.;
const X: f32 = 0.;
const Y: f32 = 0.;
const NUM_RAYS: usize = 360000;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::hex("2e2e2e").unwrap()))
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
        // Uncomment/Comment to benchmark
        // .add_startup_system(setup_system)
        .add_startup_system(setup_system_par)
        .run();
}

// ! Work with only "line" element, and only if the first 4 attributes are x1,y1,x2,y2 !
// TODO: Make it so that it actually work.
fn get_points(file: &str) -> Vec<(Vec2, Vec2)> {
    let file = fs::read_to_string(file).unwrap();
    let mut reader = Reader::from_str(&file);
    reader.trim_text(true);
    let mut points = Vec::<(Vec2, Vec2)>::new();
    loop {
        match reader.read_event_unbuffered() {
            Ok(Event::Empty(ref e)) => {
                // Assigns coordinates of points to tuple of (Vec2[y,x],Vec2[y,x])
                let values = e.attributes().map(|a| a.unwrap().value).collect::<Vec<_>>();

                points.push((
                    vec2(
                        String::from_utf8(values[1].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap()
                            - HEIGHT / 2.,
                        WIDTH / 2.
                            - String::from_utf8(values[0].clone().to_vec()).unwrap()[..]
                                .parse::<f32>()
                                .unwrap(),
                    ),
                    vec2(
                        String::from_utf8(values[3].clone().to_vec()).unwrap()[..]
                            .parse::<f32>()
                            .unwrap()
                            - HEIGHT / 2.,
                        WIDTH / 2.
                            - String::from_utf8(values[2].clone().to_vec()).unwrap()[..]
                                .parse::<f32>()
                                .unwrap(),
                    ),
                ));
            }
            Ok(Event::Eof) => break, // exits the loop when reaching end of file
            Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),
            _ => (), // There are several other `Event`s we do not consider here
        }
    }
    points
}

fn direction_to_coord(radius: u16, angle: f32) -> Vec2 {
    use std::f32::consts::PI;
    let x = (radius as f32) * f32::sin(PI * 2. * angle / 360.);
    let y = (radius as f32) * f32::cos(PI * 2. * angle / 360.);
    Vec2::new(x, y)
}

fn get_intersect(ray: (Vec2, Vec2), walls: Vec<(Vec2, Vec2)>) -> Option<Vec2> {
    let mut intersect_points = Vec::<Vec2>::new();
    walls.iter().for_each(|wall| {
        // Line–line intersection {https://en.wikipedia.org/wiki/Line%E2%80%93line_intersection}
        let (x1, x2, y1, y2) = (&ray.0[0], &ray.1[0], &ray.0[1], &ray.1[1]);
        let (x3, x4, y3, y4) = (&wall.0[0], &wall.1[0], &wall.0[1], &wall.1[1]);
        let t_denominator = (x1 - x3) * (y3 - y4) - (y1 - y3) * (x3 - x4);
        let u_denominator = (x1 - x3) * (y1 - y2) - (y1 - y3) * (x1 - x2);
        let numerator = (x1 - x2) * (y3 - y4) - (y1 - y2) * (x3 - x4);
        let t = t_denominator / numerator;
        let u = u_denominator / numerator;
        if (0. ..=1.).contains(&t) && (0. ..=1.).contains(&u) {
            intersect_points.push(Vec2::new(x1 + t * (x2 - x1), y1 + t * (y2 - y1)));
        }
    });
    // Check if there are more than 1 intersections, then compare them and pick the shortest path
    if intersect_points.is_empty() {
        None
    } else if intersect_points.len() == 1_usize {
        Some(intersect_points[0])
    } else {
        let mut distance_dict = Vec::<(Vec2, f32)>::new();
        intersect_points.iter().for_each(|wall| {
            distance_dict.push((
                *wall,
                (wall[0] - ray.0[0]).powf(2.) + (wall[1] - ray.0[1]).powf(2.),
            ));
        });
        distance_dict.sort_by_key(|k| k.1 as u32);
        let closest_intersect = distance_dict[0].0;
        Some(closest_intersect)
    }
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let now = Instant::now();

    let svg = asset_server.load("image.svg");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(Svg2dBundle {
        svg,
        origin: Origin::Center,
        ..Default::default()
    });

    let points = get_points("assets/image.svg");
    let dest_angle = lin_space(0.0..360.0, NUM_RAYS).collect::<Vec<f32>>();
    let max_size = WIDTH.max(HEIGHT) as u16;

    let mut count = 0_usize;
    let mut path_builder = PathBuilder::new();
    while count < NUM_RAYS {
        let dest_coord = direction_to_coord(max_size, dest_angle[count]);
        let final_dest = get_intersect((Vec2::new(X, Y), dest_coord), points.clone());

        path_builder.move_to(Vec2::new(X, Y));
        match final_dest {
            Some(_) => path_builder.line_to(final_dest.unwrap()),
            None => path_builder.line_to(dest_coord),
        };
        count += 1;
    }
    commands.spawn_bundle(GeometryBuilder::build_as(
        &path_builder.build(),
        DrawMode::Stroke(StrokeMode::new(Color::hex("6c99bb").unwrap(), 1.0)),
        Transform::default(),
    ));

    println!("seq= {:?}s", now.elapsed());
}

fn setup_system_par(mut commands: Commands, asset_server: Res<AssetServer>) {
    let now = Instant::now();

    let svg = asset_server.load("image.svg");
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(Svg2dBundle {
        svg,
        origin: Origin::Center,
        ..Default::default()
    });

    let points = get_points("assets/image.svg");
    let dest_angle = lin_space(0.0..360.0, NUM_RAYS).collect::<Vec<f32>>();
    let max_size = WIDTH.max(HEIGHT) as u16;

    let pool = ThreadPoolBuilder::new()
        .num_threads(num_cpus::get())
        .build()
        .unwrap();
    let mut path_builder = PathBuilder::new();
    pool.install(|| {
        for angle in dest_angle.iter().take(NUM_RAYS) {
            let dest_coord = direction_to_coord(max_size, *angle);
            let final_dest = get_intersect((Vec2::new(X, Y), dest_coord), points.clone());

            path_builder.move_to(Vec2::new(X, Y));
            match final_dest {
                Some(_) => path_builder.line_to(final_dest.unwrap()),
                None => path_builder.line_to(dest_coord),
            };
        }
    });
    commands.spawn_bundle(GeometryBuilder::build_as(
        &path_builder.build(),
        DrawMode::Stroke(StrokeMode::new(Color::hex("6c99bb").unwrap(), 1.0)),
        Transform::default(),
    ));

    println!("par= {:?}s", now.elapsed());
}
