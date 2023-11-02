use a_star_wallpaper::{
    check_for_file, create_folder_at_path, execute_osm_query, grab_city_data, random_file_in_folder,
};
use image::RgbImage;
use serde_json::Value;
use std::cmp::min;
use std::error::Error;
use std::fs;
use std::io::Write;
use std::process::Command;
use std::{collections::HashMap, fs::File, io::BufWriter};
use tempfile::NamedTempFile;
use tokio::task::spawn_blocking;
use wallpaper;

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
    let file = File::create(path_name).expect("Failed to create file");
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

fn set_wallpaper(path: &str) -> Result<(), Box<dyn Error>> {
    // Generate the Applescript string
    let cmd = &format!(
        r#"tell app "finder" to set desktop picture to POSIX file {}"#,
        enquote::enquote('"', path),
    );
    // Run it using osascript
    Command::new("osascript").args(&["-e", cmd]).output()?;

    Ok(())
}

async fn draw_city() {
    let city_data = random_city_data();
    let (min_lat, min_long, max_lat, max_long) = get_location_bounds(&city_data);
    println!(
        "min_lat: {}, min_long: {}, max_lat: {}, max_long: {}",
        min_lat, min_long, max_lat, max_long
    );
    let image_width: u32 = 1920;
    let image_height: u32 =
        ((max_lat - min_lat) / (max_long - min_long) * image_width as f64).floor() as u32;

    println!(
        "image_width: {}, image_height: {}",
        image_width, image_height
    );

    let mut imgbuf: RgbImage = image::ImageBuffer::new(image_width, image_height);
    let elements = city_data.get("elements").unwrap().as_array().unwrap();
    for element in elements {
        if element.get("type").unwrap().as_str().unwrap() != "node" {
            continue;
        }

        let lat = element.get("lat").unwrap().as_f64().unwrap();
        let long = element.get("lon").unwrap().as_f64().unwrap();

        let image_x = min(
            ((long - min_long) / (max_long - min_long) * image_width as f64).floor() as u32,
            image_width - 1,
        );

        let image_y = image_height
            - 1
            - min(
                ((lat - min_lat) / (max_lat - min_lat) * image_height as f64).floor() as u32,
                image_height - 1,
            );

        imgbuf.put_pixel(image_x, image_y, image::Rgb([255, 255, 255]));
    }

    let path = "/Users/flatypus/Documents/a_star_wallpaper/wallpaper.png";
    imgbuf.save(path).unwrap();
    wallpaper::set_from_path(path).unwrap();
}

#[tokio::main]
async fn main() {
    // grab_cities().await;
    draw_city().await;
}
