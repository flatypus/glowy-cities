use a_star_wallpaper::grab_cities;
use bevy::prelude::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    sprite::MaterialMesh2dBundle,
};
use bevy_tweening::*;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
// use wallpaper;

fn get_location_bounds(city_data: &Value) -> (f64, f64, f64, f64) {
    let nodes = city_data.get("nodes").unwrap().as_object().unwrap();
    let (mut min_lat, mut min_long, mut max_lat, mut max_long) =
        (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for (id, data) in nodes {
        let lat = data.get("lat").unwrap().as_f64().unwrap();
        let long = data.get("lon").unwrap().as_f64().unwrap();
        if lat < min_lat {
            min_lat = lat;
        }
        if lat > max_lat {
            max_lat = lat;
        }
        if long < min_long {
            min_long = long;
        }
        if long > max_long {
            max_long = long;
        }
    }
    return (min_lat, min_long, max_lat, max_long);
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut city_metadata: ResMut<CityMetadata>,
) {
    commands.spawn((
        Camera2dBundle {
            camera: Camera {
                hdr: true,
                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface,
            ..default()
        },
        BloomSettings::default(),
    ));

    let material = materials.add(ColorMaterial::from(Color::rgba(14.0, 2.0, 0.0, 0.5)));
    city_metadata.material = material;
}

#[derive(Copy, Clone)]
struct RoadSegment {
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    angle: f32,
    length: f32,
}

fn setup_city(search: &str) -> CityMetadata {
    let city_data = a_star_wallpaper::choose_city(search);
    let bounds = get_location_bounds(&city_data);
    let (min_lat, min_long, max_lat, max_long) = bounds;
    let image_width: f32 = 1920.0;
    let image_height: f32 =
        ((max_lat - min_lat) / (max_long - min_long) * image_width as f64).floor() as f32;

    let nodes = city_data.get("nodes").unwrap().as_object().unwrap();
    let ways = city_data.get("ways").unwrap().as_array().unwrap();

    const SCALE: f32 = 0.5;

    let mut road_segments = Vec::new();

    for way in ways {
        let mut points = Vec::new();
        for node_id in way.as_array().unwrap() {
            let node = nodes.get(node_id.as_str().unwrap()).unwrap();
            let lat = node.get("lat").unwrap().as_f64().unwrap();
            let long = node.get("lon").unwrap().as_f64().unwrap();
            let image_x = (((long - min_long) / (max_long - min_long) * image_width as f64).floor()
                as i32
                - image_width as i32 / 2) as f32;
            let image_y = (((lat - min_lat) / (max_lat - min_lat) * image_height as f64).floor()
                as i32
                - image_height as i32 / 2) as f32;
            points.push(Vec2::new(image_x, image_y));
        }

        for i in 0..points.len() - 1 {
            let start_x = points[i].x * SCALE;
            let start_y = points[i].y * SCALE;
            let end_x = points[i + 1].x * SCALE;
            let end_y = points[i + 1].y * SCALE;

            let angle = (end_y - start_y).atan2(end_x - start_x);
            let length = ((end_y - start_y).powi(2) + (end_x - start_x).powi(2)).sqrt();

            road_segments.push(RoadSegment {
                start_x,
                start_y,
                end_x,
                end_y,
                angle,
                length,
            });
        }
    }

    CityMetadata {
        nodes: nodes.clone(),
        ways: ways.clone(),
        bounds,
        road_segments,
        image_width,
        image_height,
        material: Handle::default(),
        mesh_cache: HashMap::new(),
    }
}

fn draw_roads(
    mut next_state: ResMut<NextState<AppMode>>,
    mut counters: ResMut<Counters>,
    mut city_metadata: ResMut<CityMetadata>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if counters.city_frame >= city_metadata.road_segments.len() as u32 {
        next_state.set(AppMode::PickPoints);
        return;
    }

    const ROADS_PER_FRAME: u32 = 1000;

    for index in counters.city_frame
        ..std::cmp::min(
            city_metadata.road_segments.len() as u32,
            counters.city_frame + ROADS_PER_FRAME,
        )
    {
        let segment = city_metadata.road_segments[index as usize];
        let rounded = (segment.length * 10.0).round() / 10.0;
        let str_length = format!("{}", rounded);
        if !city_metadata.mesh_cache.contains_key(&str_length) {
            city_metadata.mesh_cache.insert(
                str_length.clone(),
                meshes
                    .add(shape::Quad::new(Vec2::new(rounded, 0.5)).into())
                    .into(),
            );
        }

        println!("Segment: {:?}", segment.length);

        commands.spawn(MaterialMesh2dBundle {
            mesh: bevy::sprite::Mesh2dHandle(
                city_metadata.mesh_cache.get(&str_length).unwrap().clone(),
            ),
            material: city_metadata.material.clone(),
            transform: Transform::from_translation(Vec3::new(
                (segment.end_x + segment.start_x) / 2.0,
                (segment.end_y + segment.start_y) / 2.0,
                0.0,
            ))
            .mul_transform(Transform::from_rotation(Quat::from_rotation_z(
                segment.angle,
            ))),
            ..default()
        });

        counters.city_frame += 1;
    }
}

fn increment_glow(mut counters: ResMut<Counters>) {
    println!("Glow: {}", counters.glow_amount);
    // change the rgb values of the glow material

    counters.glow_amount += 1;
}

fn track_fps(time: Res<Time>, mut counters: ResMut<Counters>) {
    println!("FPS: {}", 1.0 / time.delta_seconds());
}

#[derive(Resource)]
struct Counters {
    city_frame: u32,
    glow_amount: u32,
}
#[derive(Resource)]
struct CityMetadata {
    nodes: serde_json::Map<String, Value>,
    ways: Vec<Value>,
    bounds: (f64, f64, f64, f64),
    image_width: f32,
    image_height: f32,
    road_segments: Vec<RoadSegment>,
    material: Handle<ColorMaterial>,
    mesh_cache: HashMap<String, Handle<Mesh>>,
}

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash, States)]
enum AppMode {
    #[default]
    DrawCity,
    PickPoints,
    RunAStar,
    DrawPath,
}

#[tokio::main]
async fn main() {
    // wallpaper::set_from_path("/Users/flatypus/Documents/excalidraw.png");
    // println!("{:?}", wallpaper::get());
    // grab_cities("Kyoto").await;

    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Counters {
            city_frame: 0,
            glow_amount: 0,
        })
        .insert_resource(setup_city("京都"))
        .add_plugins(DefaultPlugins)
        .add_plugins(TweeningPlugin)
        .add_state::<AppMode>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw_roads.run_if(in_state(AppMode::DrawCity)))
        .add_systems(Update, increment_glow.run_if(in_state(AppMode::PickPoints)))
        .add_systems(Update, track_fps)
        .run();
}
