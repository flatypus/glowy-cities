use a_star_wallpaper::grab_cities;
use bevy::prelude::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    sprite::MaterialMesh2dBundle,
};
use serde_json::Value;

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

fn setup(mut commands: Commands) {
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
}

fn setup_city() -> CityMetadata {
    let city_data = a_star_wallpaper::random_city_data();
    let bounds = get_location_bounds(&city_data);
    let (min_lat, min_long, max_lat, max_long) = bounds;
    let image_width: f32 = 1920.0;
    let image_height: f32 =
        ((max_lat - min_lat) / (max_long - min_long) * image_width as f64).floor() as f32;
    let nodes = city_data.get("nodes").unwrap().as_object().unwrap();
    let ways = city_data.get("ways").unwrap().as_object().unwrap();
    return CityMetadata {
        nodes: nodes.clone(),
        ways: ways.clone(),
        bounds,
        image_width,
        image_height,
    };
}

fn draw_cities(
    mut next_state: ResMut<NextState<AppMode>>,
    mut counters: ResMut<Counters>,
    city_metadata: ResMut<CityMetadata>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let bounds = &city_metadata.bounds;
    let image_width = city_metadata.image_width;
    let image_height = city_metadata.image_height;
    let mesh: bevy::sprite::Mesh2dHandle =
        meshes.add(shape::RegularPolygon::new(0.4, 4).into()).into();
    let material = materials.add(ColorMaterial::from(Color::rgb(14.0, 2.0, 0.0)));
    let nodes = &city_metadata.nodes;
    let ways = &city_metadata.ways;

    if counters.city_frame > nodes.len() as u32 {
        next_state.set(AppMode::PickPoints);
    }

    // convert node object to id list from keys
    let key_list = nodes.keys().collect::<Vec<&String>>();

    for item in counters.city_frame..std::cmp::min(nodes.len() as u32, counters.city_frame + 100000)
    {
        let id = key_list[item as usize];
        let node = nodes.get(id).unwrap();
        let lat = node.get("lat").unwrap().as_f64().unwrap();
        let long = node.get("lon").unwrap().as_f64().unwrap();

        let (min_lat, min_long, max_lat, max_long) = bounds;

        let image_x = (((long - min_long) / (max_long - min_long) * image_width as f64).floor()
            as i32)
            - (image_width as i32 / 2);

        let image_y = (((lat - min_lat) / (max_lat - min_lat) * image_height as f64).floor()
            as i32)
            - (image_height as i32 / 2);

        println!(
            "lat: {}, long: {}, image_x: {}, image_y: {}, image_width:{}, image_height:{}, boxes: {}",
            lat, long, image_x, image_y, image_width, image_height, counters.city_frame
        );

        commands.spawn(MaterialMesh2dBundle {
            mesh: mesh.clone(),
            material: material.clone(),
            transform: Transform::from_translation(Vec3::new(
                image_x as f32 * 0.6,
                image_y as f32 * 0.6,
                0.0,
            )),
            ..default()
        });

        counters.city_frame += 1;
    }
}

#[derive(Resource)]
struct Counters {
    city_frame: u32,
}
#[derive(Resource)]
struct CityMetadata {
    nodes: serde_json::Map<String, Value>,
    ways: serde_json::Map<String, Value>,
    bounds: (f64, f64, f64, f64),
    image_width: f32,
    image_height: f32,
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
    grab_cities("Vancouver").await;
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Counters { city_frame: 0 })
        .insert_resource(setup_city())
        .add_plugins(DefaultPlugins)
        .add_state::<AppMode>()
        .add_systems(Startup, setup)
        .add_systems(Update, draw_cities.run_if(in_state(AppMode::DrawCity)))
        .run();
}
