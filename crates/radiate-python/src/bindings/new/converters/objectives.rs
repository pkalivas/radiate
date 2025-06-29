use crate::{InputConverter, PyEngineInput, PyEngineInputType};
use radiate::*;

impl<C> InputConverter<C, Objective> for PyEngineInput
where
    C: Chromosome,
{
    fn convert(&self) -> Objective {
        if self.input_type != PyEngineInputType::Objective {
            panic!("Input type {:?} not an objective", self.input_type);
        }

        let objectives = self.args.get("objectives").map(|objs| {
            objs.split('|')
                .map(|s| match s.trim().to_lowercase().as_str() {
                    "min" => Optimize::Minimize,
                    "max" => Optimize::Maximize,
                    _ => panic!("Objective {} not recognized", s),
                })
                .collect::<Vec<Optimize>>()
        });

        match objectives {
            Some(objs) => {
                if objs.len() == 1 {
                    return Objective::Single(objs[0]);
                } else if objs.len() > 1 {
                    return Objective::Multi(objs);
                } else {
                    panic!("No objectives provided");
                }
            }
            None => Objective::Single(Optimize::Maximize),
        }
    }
}
