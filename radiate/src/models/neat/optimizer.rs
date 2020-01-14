


#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum Optimizer {
    SGD(f32),
    ADAM(f32, u32)
}



impl Optimizer {



}