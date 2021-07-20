pub mod dense;
pub mod gru;
pub mod layer;
pub mod lstm;
pub mod vectorops;

pub mod layertype {

    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
    pub enum LayerType {
        DensePool,
        Dense,
        LSTM,
        GRU,
    }
}
