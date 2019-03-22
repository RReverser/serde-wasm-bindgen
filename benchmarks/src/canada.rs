use serde::{Deserialize, Serialize};
use std::collections::BTreeMap as Map;

pub type Canada = FeatureCollection;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FeatureCollection {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub features: Vec<Feature>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feature {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub properties: Map<String, String>,
    pub geometry: Geometry,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Geometry {
    #[serde(rename = "type")]
    pub obj_type: ObjType,
    pub coordinates: Vec<Vec<(Latitude, Longitude)>>,
}

pub type Latitude = f32;
pub type Longitude = f32;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum ObjType {
    FeatureCollection,
    Feature,
    Polygon,
}
