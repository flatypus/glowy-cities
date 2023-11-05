use a_star_wallpaper::{
    check_for_file, create_folder_at_path, execute_osm_query, grab_city_data, random_file_in_folder,
};
use bevy::prelude::*;
use bevy::{
    core_pipeline::{bloom::BloomSettings, tonemapping::Tonemapping},
    sprite::MaterialMesh2dBundle,
};
use serde_json::Value;
use std::cmp::min;
use std::fs;
use std::{collections::HashMap, io::BufWriter};

fn get_query(area_id: &str) -> String {
    return format!("[timeout:900][out:json];\
        area({});\
        (._; )->.area;\
        (\
        way[highway~'^(((motorway|trunk|primary|secondary|tertiary)(_link)?)|unclassified|residential|living_street|service|track)$'](area.area);\
        node(w);\
        );\
        out skel;", area_id);
}

fn save_file(path_data: Value, path_name: &str) -> Result<(), ()> {
    let file = fs::File::create(path_name).expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &path_data).expect("Failed to write file");
    Ok(())
}

async fn query_overpass(place: HashMap<&str, Value>) {
    let area_id = place.get("area_id").unwrap().as_f64().unwrap().to_string();
    let city_name = place.get("key").unwrap().as_str().unwrap();
    let query = get_query(&area_id);
    println!("{}", query);

    create_folder_at_path("tmp/overpass");
    let path = format!("tmp/overpass/{}_{}.json", city_name, area_id);
    if check_for_file(path.as_str()) {
        println!("File {} already exists", path);
        return;
    }
    let query_result = execute_osm_query(&query).await;
    save_file(query_result, &path).expect("Failed to save file");
}

async fn grab_cities() {
    let city = "Vancouver";
    let city_data = grab_city_data(city)
        .await
        .expect("Failed to grab city data");

    // loop over run async
    for place in city_data {
        query_overpass(place).await;
    }
}

fn random_city_data() -> Value {
    let city_path = random_file_in_folder("tmp/overpass");
    println!("City_path: {}", city_path);
    let city_data = serde_json::from_str::<Value>(&std::fs::read_to_string(city_path).unwrap())
        .expect("Failed to parse JSON");
    return city_data;
}

fn get_location_bounds(city_data: &Value) -> (f64, f64, f64, f64) {
    let elements = city_data.get("elements").unwrap().as_array().unwrap();
    let (mut min_lat, mut min_long, mut max_lat, mut max_long) =
        (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for element in elements {
        if element.get("type").unwrap().as_str().unwrap() != "node" {
            continue;
        }
        let lat = element.get("lat").unwrap().as_f64().unwrap();
        let long = element.get("lon").unwrap().as_f64().unwrap();
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
    let city_data = random_city_data();
    let bounds = get_location_bounds(&city_data);
    let (min_lat, min_long, max_lat, max_long) = bounds;
    let image_width: f32 = 1920.0;
    let image_height: f32 =
        ((max_lat - min_lat) / (max_long - min_long) * image_width as f64).floor() as f32;
    return CityMetadata {
        city_data,
        bounds,
        image_width,
        image_height,
    };
}

fn add_city(
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

    let elements = city_metadata
        .city_data
        .get("elements")
        .unwrap()
        .as_array()
        .unwrap();

    if counters.city_frame > elements.len() as u32 {
        next_state.set(AppMode::PickPoints);
    }

    for item in counters.city_frame..min(elements.len() as u32, counters.city_frame + 10000) {
        let element = &elements[item as usize];

        if element.get("type").unwrap().as_str().unwrap() != "node" {
            counters.city_frame += 1;
            return;
        }

        let lat = element.get("lat").unwrap().as_f64().unwrap();
        let long = element.get("lon").unwrap().as_f64().unwrap();
        let (min_lat, min_long, max_lat, max_long) = bounds;

        let image_x = (((long - min_long) / (max_long - min_long) * image_width as f64).floor()
            as i32)
            - (image_width as i32 / 2);

        let image_y = (((lat - min_lat) / (max_lat - min_lat) * image_height as f64).floor()
            as i32)
            - (image_height as i32 / 2);

        // println!(
        //     "lat: {}, long: {}, image_x: {}, image_y: {}, image_width:{}, image_height:{}, boxes: {}",
        //     lat, long, image_x, image_y, image_width, image_height, counters.city_frame
        // );

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
    city_data: Value,
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
    // grab_cities().await;
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Counters { city_frame: 0 })
        .insert_resource(setup_city())
        .add_plugins(DefaultPlugins)
        .add_state::<AppMode>()
        .add_systems(Startup, setup)
        .add_systems(Update, add_city.run_if(in_state(AppMode::DrawCity)))
        .run();
}
