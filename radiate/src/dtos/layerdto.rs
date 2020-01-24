

use super::{
    activationdto::ActivationDto,
    layertypedto::LayerTypeDto
};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDto {
    pub num_in: u32,
    pub num_out: u32,
    pub memory_size: i32,
    pub layer_type: LayerTypeDto,
    pub activation: ActivationDto
}
