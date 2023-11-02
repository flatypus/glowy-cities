use reqwest;
use serde_json::{json, Value};
use std::collections::HashMap;

async fn grab_city_data(city: &str) -> Result<Vec<HashMap<&str, Value>>, ()> {
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
        .expect("Failed to fetch");

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
                    "areaId",
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

#[tokio::main]
async fn main() {
    let city = "Vancouver";
    let city_data = grab_city_data(city)
        .await
        .expect("Failed to grab city data");
    println!("{:?}", city_data);
}
