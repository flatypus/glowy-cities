use rand::seq::IteratorRandom;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;

pub async fn grab_city_data(city: &str) -> Result<Vec<HashMap<&str, Value>>, ()> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://nominatim.openstreetmap.org/search?format=json&q={}",
        city
    );
    let request = client
        .get(url)
        .header("User-Agent", "reqwest")
        .header("Accept", "application/json")
        .send()
        .await
        .expect("Failed to fetch nominatim");

    let json_value = request
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON");

    if let Some(json_array) = json_value.as_array() {
        let cities: Vec<HashMap<&str, Value>> = json_array
            .iter()
            .filter(|row| {
                row.get("osm_type")
                    .map_or(false, |osm_type| osm_type == "relation")
            })
            .map(|row| {
                let mut map: HashMap<&str, Value> = HashMap::new();
                map.insert("key", json!(city));
                map.insert("name", row.get("display_name").unwrap().clone());
                map.insert("type", row.get("type").unwrap().clone());
                map.insert(
                    "area_id",
                    json!(row.get("osm_id").unwrap().as_f64().unwrap() + 36e8),
                );
                return map;
            })
            .collect();
        // throw if no city found
        if cities.len() == 0 {
            return Err(());
        }
        return Ok(cities);
    }
    return Err(());
}

pub async fn execute_osm_query(query: &str) -> Value {
    let client = reqwest::Client::new();
    const OVERPASS: &str = "https://overpass-api.de/api/interpreter";

    println!("{}", query);

    let request = client
        .get(OVERPASS)
        .body(format!("data={}", query))
        .header("User-Agent", "reqwest")
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .expect("Failed to fetch overpass query");

    let json_value = request
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse JSON");

    return json_value;
}

pub fn create_folder_at_path(path: &str) {
    std::fs::create_dir_all(path).expect("Failed to create folder");
}

pub fn check_for_file(path: &str) -> bool {
    return std::path::Path::new(path).exists();
}

pub fn random_file_in_folder(path: &str) -> String {
    let mut rng = rand::thread_rng();
    if !check_for_file(path) {
        panic!("Folder {} does not exist", path);
    }
    let files = fs::read_dir(path).unwrap();
    let file = files.choose(&mut rng).unwrap().unwrap();
    return file.path().to_str().unwrap().to_string();
}
