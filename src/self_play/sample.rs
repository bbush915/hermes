use serde::Serialize;

#[derive(Serialize)]
pub struct Sample {
    pub state: Vec<f32>,
    pub policy: Vec<f32>,
    pub value: f32,
}
