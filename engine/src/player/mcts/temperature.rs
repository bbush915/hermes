#[derive(Clone)]
pub enum TemperatureSchedule {
    Constant(f32),
    Step { threshold: u32, hi: f32, lo: f32 },
    Linear { threshold: u32, hi: f32, lo: f32 },
}

impl TemperatureSchedule {
    pub fn get_temperature(&self, turn_number: u32) -> f32 {
        match self {
            TemperatureSchedule::Constant(temperature) => *temperature,
            TemperatureSchedule::Step { threshold, hi, lo } => {
                if turn_number < *threshold {
                    *hi
                } else {
                    *lo
                }
            }
            TemperatureSchedule::Linear { threshold, hi, lo } => {
                if turn_number >= *threshold {
                    *lo
                } else {
                    let t = turn_number as f32 / *threshold as f32;

                    hi - (hi - lo) * t
                }
            }
        }
    }
}
