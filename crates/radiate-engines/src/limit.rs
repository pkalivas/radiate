use radiate_core::Score;
use std::time::Duration;

#[derive(Debug, Clone)]
pub enum Limit {
    Generation(usize),
    Seconds(Duration),
    Score(Score),
    Convergence(usize, f32),
    Combined(Vec<Limit>),
}

impl Into<Limit> for usize {
    fn into(self) -> Limit {
        Limit::Generation(self)
    }
}

impl Into<Limit> for Duration {
    fn into(self) -> Limit {
        Limit::Seconds(self)
    }
}

impl Into<Limit> for f32 {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for Vec<f32> {
    fn into(self) -> Limit {
        Limit::Score(Score::from(self))
    }
}

impl Into<Limit> for (usize, f32) {
    fn into(self) -> Limit {
        Limit::Convergence(self.0, self.1)
    }
}

impl Into<Limit> for Vec<Limit> {
    fn into(self) -> Limit {
        Limit::Combined(self)
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_limit_conversions() {
        use super::Limit;
        use std::time::Duration;

        let gen_limit: Limit = 100.into();
        match gen_limit {
            Limit::Generation(n) => assert_eq!(n, 100),
            _ => panic!("Expected Generation limit"),
        }

        let time_limit: Limit = Duration::from_secs(60).into();
        match time_limit {
            Limit::Seconds(dur) => assert_eq!(dur, Duration::from_secs(60)),
            _ => panic!("Expected Seconds limit"),
        }

        let score_limit: Limit = 95.5f32.into();
        match score_limit {
            Limit::Score(score) => assert_eq!(score.as_f32(), 95.5),
            _ => panic!("Expected Score limit"),
        }

        let multi_score_limit: Limit = vec![90.0f32, 85.5f32, 78.0f32].into();
        match multi_score_limit {
            Limit::Score(score) => {
                assert_eq!(score[0], 90.0);
                assert_eq!(score[1], 85.5);
                assert_eq!(score[2], 78.0);
            }
            _ => panic!("Expected Multi Score limit"),
        }

        let conv_limit: Limit = (10, 0.01f32).into();
        match conv_limit {
            Limit::Convergence(gens, thresh) => {
                assert_eq!(gens, 10);
                assert_eq!(thresh, 0.01);
            }
            _ => panic!("Expected Convergence limit"),
        }

        let generation_combined_limit: Limit = 100.into();
        let duration_combined_limit: Limit = Duration::from_secs(30).into();
        let combined_limit: Limit = vec![generation_combined_limit, duration_combined_limit].into();
        match combined_limit {
            Limit::Combined(limits) => assert_eq!(limits.len(), 2),
            _ => panic!("Expected Combined limit"),
        }
    }
}
