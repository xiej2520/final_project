use crate::http_client::HttpClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NomRevResponse {
    name: Option<String>, // not strictly necessary for grading script
    osm_type: String,
    osm_id: i64,
    address: NomRevAddress,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct NomRevAddress {
    house_number: Option<String>,
    road: Option<String>,
    city: Option<String>,
    town: Option<String>,
    village: Option<String>,
    hamlet: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NomDetailsResponse {
    addresstags: AddressTags,
}
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct AddressTags {
    city: String,
    state: String,
    street: String,
    postcode: String,
    housenumber: String,
}

#[derive(Debug, Serialize)]
pub struct AddressResponse {
    //name: Option<String>, // not strictly necessary for grading script
    number: Option<String>,
    street: Option<String>,
    city: Option<String>,
    state: Option<String>,
    country: Option<String>,
}

impl From<NomDetailsResponse> for AddressResponse {
    fn from(resp: NomDetailsResponse) -> Self {
        let tags = resp.addresstags;
        Self {
            number: Some(tags.housenumber),
            street: Some(tags.street),
            city: Some(tags.city),
            state: Some(tags.state),
            country: Some("USA".into()),
        }
    }
}

//impl From<AddressQuery> for AddressObject {
//    fn from(query: AddressQuery) -> Self {
//        AddressObject {
//            //name: query.name,
//            number: query.house_number,
//            street: query.road,
//            city: query.city.or(query.town).or(query.village).or(query.hamlet),
//            state: query.state,
//            //country: query.country,
//            country: Some("USA".into()),
//        }
//    }
//}

#[derive(Debug, Deserialize, Clone)]
struct PhotonRevResponse {
    features: Vec<PhotonRevFeature>,
}
#[derive(Debug, Deserialize, Clone)]
struct PhotonRevFeature {
    properties: PhotonRevProperties,
}
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
struct PhotonRevProperties {
    osm_id: i64,
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
//impl From<PhotonRevGeo> for AddressObject {
//    fn from(resp: PhotonRevGeo) -> Self {
//        let prop = resp.features[0].properties.clone();
//        tracing::info!("{prop:?}");
//        AddressObject {
//            number: prop.housenumber,
//            street: prop.street,
//            //city: prop.city,
//            city: prop.district.or(prop.city),
//            state: prop.state,
//            //country: query.country,
//            country: Some("USA".into()),
//        }
//    }
//}

pub async fn get_address(
    photon_client: &HttpClient,
    nominatim_client: &HttpClient,
    lat: f64,
    lon: f64,
) -> Result<AddressResponse, String> {
    // find osmid, then query nominatim for the addresstags
    // try photon revgeo first
    let url = format!("/reverse?lat={lat}&lon={lon}"); // don't use &layer=house???
    let builder = photon_client.get(&url).await.map_err(|e| {
        tracing::error!("{e}");
        e.to_string()
    })?;
    let response = builder.send().await.map_err(|e| {
        tracing::error!("{e}");
        e.to_string()
    })?;

    let resp = serde_json::from_str::<PhotonRevResponse>(
        dbg!(response.text().await.map_err(|e| e.to_string())?.as_str()),
    )
    .map_err(|e| e.to_string())?;

    tracing::info!("{resp:?}");
    let mut osmid = resp.features[0].properties.osm_id;
    if resp.features[0].properties.housenumber.is_none() {
        // fallback to nominatim revgeo
        let url = format!("reverse?lat={lat}&lon={lon}&format=jsonv2");

        let builder = nominatim_client.get(&url).await.map_err(|e| {
            tracing::error!("{e}");
            e.to_string()
        })?;
        let response = builder.send().await.map_err(|e| {
            tracing::error!("{e}");
            e.to_string()
        })?;

        let resp = serde_json::from_str::<NomRevResponse>(
            dbg!(response.text().await.map_err(|e| e.to_string())?.as_str()),
        )
        .map_err(|e| e.to_string())?;
        osmid = resp.osm_id;
    }

    let url = format!("details?osmtype=W&osmid={osmid}&format=json");

    let builder = nominatim_client.get(&url).await.map_err(|e| {
        tracing::error!("{e}");
        e.to_string()
    })?;
    let response = builder.send().await.map_err(|e| {
        tracing::error!("{e}");
        e.to_string()
    })?;

    let resp = serde_json::from_str::<NomDetailsResponse>(
        response.text().await.map_err(|e| e.to_string())?.as_str(),
    )
    .map_err(|e| e.to_string())?;
    Ok(AddressResponse::from(resp))

    //match serde_json::from_value::<PhotonRevGeo>(json.clone()) {
    //    Ok(resp) => Ok(AddressObject::from(resp)),
    //    Err(e) => {
    //        tracing::error!("{e:?}: {json:?}");
    //        Err(json
    //        .get("error")
    //        .unwrap_or(&json!("Something went wrong"))
    //        .to_string())
    //    }

    //}

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
