use serde::{Deserialize, Serialize};
use serde_json::json;
use server::http_client::HttpClient;

#[derive(Debug, Deserialize)]
struct AddressQuery {
    house_number: Option<String>,
    road: Option<String>,
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AddressObject {
    number: Option<String>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

impl From<AddressQuery> for AddressObject {
    fn from(query: AddressQuery) -> Self {
        AddressObject {
            number: query.house_number,
            street: query.road,
            city: query.city.or(query.town).or(query.village).or(query.hamlet),
            state: query.state,
            country: query.country,
        }
    }
}

pub async fn get_address(client: &HttpClient, lat: f64, lon: f64) -> Result<AddressObject, String> {
    let url = format!("reverse?lat={lat}&lon={lon}&format=jsonv2");

    let builder = match client.get(&url).await {
        Ok(builder) => builder,
        Err(e) => return Err(e.to_string()),
    };
    let response = match builder.send().await {
        Ok(response) => response,
        Err(e) => return Err(e.to_string()),
    };
    let json: serde_json::Value = match response.json().await {
        Ok(json) => json,
        Err(e) => return Err(e.to_string()),
    };

    match json.get("address") {
        Some(address) => {
            let address: AddressQuery = match serde_json::from_value(address.clone()) {
                Ok(address) => address,
                Err(e) => return Err(e.to_string()),
            };
            Ok(AddressObject::from(address))
        }
        None => Err(json
            .get("error")
            .unwrap_or(&json!("Something went wrong"))
            .to_string()),
    }
}
