use a_star_wallpaper::{check_for_file, create_folder_at_path, execute_osm_query, grab_city_data};
use serde_json::Value;
use std::{collections::HashMap, fs::File, io::BufWriter};

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
    create_folder_at_path("tmp");
    let path = format!("tmp/{}_{}.json", city_name, area_id);
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

#[tokio::main]
async fn main() {
    grab_cities().await;
}
