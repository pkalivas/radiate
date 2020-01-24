
use super::layerdto::LayerDto;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeatDto {
    pub input_size: u32,
    pub batch_size: u32,
    pub layers: Vec<LayerDto>
}
