use crate::PyEngineParam;

const SECONDS_LIMIT: &str = "seconds";
const GENERATIONS_LIMIT: &str = "generations";
const SCORE_LIMIT: &str = "score";

pub enum Limit {
    Seconds(f64),
    Generations(usize),
    Score(f32),
}

impl Limit {
    pub fn seconds(seconds: f64) -> Self {
        Limit::Seconds(seconds)
    }

    pub fn generations(generations: usize) -> Self {
        Limit::Generations(generations)
    }

    pub fn score(score: f32) -> Self {
        Limit::Score(score)
    }
}

impl From<PyEngineParam> for Limit {
    fn from(param: PyEngineParam) -> Self {
        match param.name() {
            SECONDS_LIMIT => Limit::seconds(
                param
                    .get_args()
                    .get("seconds")
                    .map(|s| s.parse::<f64>().unwrap())
                    .unwrap_or(0.0),
            ),
            GENERATIONS_LIMIT => Limit::generations(
                param
                    .get_args()
                    .get("generations")
                    .map(|s| s.parse::<usize>().unwrap())
                    .unwrap_or(0),
            ),
            SCORE_LIMIT => Limit::score(
                param
                    .get_args()
                    .get("score")
                    .map(|s| s.parse::<f32>().unwrap())
                    .unwrap_or(0.0),
            ),
            _ => panic!("Invalid limit type"),
        }
    }
}
