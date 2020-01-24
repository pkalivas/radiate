pub mod neatenvdto;
pub mod layerdto;
pub mod neatdto;

pub mod activationdto {

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ActivationDto {
        pub activation: i32,
        pub value: f32
    }

}


pub mod layertypedto {

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LayerTypeDto {
        pub layer_type: i32,
    }

}