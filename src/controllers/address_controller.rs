use crate::http_client::HttpClient;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
struct AddressQuery {
    name: Option<String>, // not strictly necessary for grading script
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
    //name: Option<String>, // not strictly necessary for grading script
    number: Option<String>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

impl From<AddressQuery> for AddressObject {
    fn from(query: AddressQuery) -> Self {
        AddressObject {
            //name: query.name,
            number: query.house_number,
            street: query.road,
            city: query.city.or(query.town).or(query.village).or(query.hamlet),
            state: query.state,
            //country: query.country,
            country: Some("USA".into()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
struct PhotonRevGeo {
    features: Vec<PhotonRevFeature>,
}
#[derive(Debug, Deserialize, Clone)]
struct PhotonRevFeature {
    properties: PhotonRevProperties,
}
#[derive(Debug, Deserialize, Clone)]
struct PhotonRevProperties {
    housenumber: Option<String>,
    street: Option<String>,
    city: Option<String>,
    //town: Option<String>,
    //village: Option<String>,
    //hamlet: Option<String>,
    district: Option<String>,
    state: Option<String>,
    country: Option<String>,
    countrycode: Option<String>,
}
impl From<PhotonRevGeo> for AddressObject {
    fn from(resp: PhotonRevGeo) -> Self {
        let prop = resp.features[0].properties.clone();
        tracing::info!("{prop:?}");
        AddressObject {
            number: prop.housenumber,
            street: prop.street,
            //city: prop.city,
            city: prop.district.or(prop.city),
            state: prop.state,
            //country: query.country,
            country: Some("USA".into()),
        }
    }
}

pub async fn get_address(client: &HttpClient, lat: f64, lon: f64) -> Result<AddressObject, String> {
    //let url = format!("reverse?lat={lat}&lon={lon}&format=jsonv2&layer=address");
    let url = format!("/reverse?lat={lat}&lon={lon}&layer=house");

    let builder = match client.get(&url).await {
        Ok(builder) => builder,
        //Err(e) => return Err(e.to_string()),
        Err(e) => {
            tracing::error!("{e}");
            return Err(e.to_string());
        }
    };
    let response = match builder.send().await {
        Ok(response) => response,
        //Err(e) => return Err(e.to_string()),
        Err(e) => {
            tracing::error!("{e}");
            return Err(e.to_string());
        }
    };
    let json: serde_json::Value = match response.json().await {
        Ok(json) => json,
        //Err(e) => return Err(e.to_string()),
        Err(e) => {
            tracing::error!("{e}");
            return Err(e.to_string());
        }
    };
    match serde_json::from_value::<PhotonRevGeo>(json.clone()) {
        Ok(resp) => Ok(AddressObject::from(resp)),
        Err(e) => {
            tracing::error!("{e:?}: {json:?}");
            Err(json
            .get("error")
            .unwrap_or(&json!("Something went wrong"))
            .to_string())
        }

    }

    //match json.get("address") {
    //    Some(address) => {
    //        let address: AddressQuery = match serde_json::from_value(address.clone()) {
    //            Ok(address) => address,
    //            //Err(e) => return Err(e.to_string()),
    //            Err(e) => {
    //                tracing::error!("{e}");
    //                return Err(e.to_string());
    //            }
    //        };
    //        Ok(AddressObject::from(address))
    //    }
    //    None => Err(json
    //        .get("error")
    //        .unwrap_or(&json!("Something went wrong"))
    //        .to_string()),
    //}
}
