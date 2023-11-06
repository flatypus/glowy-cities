use rand::seq::IteratorRandom;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::BufWriter;

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

    println!("Querying overpass for: {}", query);

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
    if files.count() as i32 == 0 {
        panic!("Folder {} is empty", path);
    }
    let file = fs::read_dir(path)
        .unwrap()
        .choose(&mut rng)
        .unwrap()
        .unwrap();
    return file.path().to_str().unwrap().to_string();
}

pub fn get_query(area_id: &str) -> String {
    return format!("[timeout:900][out:json];\
        area({});\
        (._; )->.area;\
        (\
        way[highway~'^(((motorway|trunk|primary|secondary|tertiary)(_link)?)|unclassified|residential|living_street|service|track)$'](area.area);\
        node(w);\
        );\
        out skel;", area_id);
}

pub fn save_file(path_data: Value, path_name: &str) -> Result<(), ()> {
    let file = fs::File::create(path_name).expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, &path_data).expect("Failed to write file");
    Ok(())
}

pub async fn query_overpass(place: HashMap<&str, Value>) {
    let area_id = place.get("area_id").unwrap().as_f64().unwrap().to_string();
    let city_name = place.get("name").unwrap().as_str().unwrap();
    let query = get_query(&area_id);

    create_folder_at_path("tmp/overpass");
    let path = format!("tmp/overpass/{}_{}.json", city_name, area_id);
    if check_for_file(path.as_str()) {
        println!("File {} already exists", path);
        return;
    }
    let query_result = execute_osm_query(&query).await;
    let elements = query_result.get("elements").unwrap().as_array().unwrap();
    let mut node_map: HashMap<String, Value> = HashMap::new();
    let mut ways: Vec<Vec<String>> = Vec::new();

    for element in elements {
        if element.get("type").unwrap().as_str().unwrap() != "node" {
            continue;
        }
        let id = element.get("id").unwrap().as_i64().unwrap().to_string();
        node_map.insert(id, {
            let mut map: HashMap<&str, Value> = HashMap::new();
            map.insert("lat", element.get("lat").unwrap().clone());
            map.insert("lon", element.get("lon").unwrap().clone());
            json!(map)
        });
    }

    for element in elements {
        if element.get("type").unwrap().as_str().unwrap() != "way" {
            continue;
        }
        ways.push(
            element
                .get("nodes")
                .unwrap()
                .as_array()
                .unwrap()
                .iter()
                .map(|node| node.as_i64().unwrap().to_string())
                .collect::<Vec<String>>(),
        );
    }

    save_file(
        {
            let mut map: HashMap<&str, Value> = HashMap::new();
            map.insert("nodes", json!(node_map));
            map.insert("ways", json!(ways));
            json!(map)
        },
        &path,
    )
    .expect("Failed to save file");
}

pub async fn grab_cities(city: &str) {
    let city_data = grab_city_data(city)
        .await
        .expect("Failed to grab city data");

    // loop over run async
    for place in city_data {
        query_overpass(place).await;
    }
}

pub fn random_city_data() -> Value {
    let city_path = random_file_in_folder("tmp/overpass");
    println!("City_path: {}", city_path);
    let city_data = serde_json::from_str::<Value>(&std::fs::read_to_string(city_path).unwrap())
        .expect("Failed to parse JSON");
    return city_data;
}
