use serde::{Deserialize, Serialize};
use heapless::Vec;


#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub view_update_every: u32,
    pub update_every: u32,
    pub first_entry: u32,
    pub last_entry: u32,
    pub before: u32,
    pub after: u32,
    pub latest_values: Vec<f32, 2>,
    pub view_latest_values: Vec<f32, 2>,
    pub dimensions: u32,
    pub points: u32,
    pub result: DataResult,
    pub min: f32,
    pub max: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DataResult {
    pub data: Vec<[f32; 3], 30>,
}
