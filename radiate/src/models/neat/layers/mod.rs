pub mod layer;
pub mod dense;
pub mod lstm;
pub mod vectorops;
pub mod gru;


pub mod layertype {


    #[derive(Deserialize, Serialize, Debug, PartialEq, Clone, Copy)]
    pub enum LayerType {
        DensePool,
        Dense,
        LSTM,
        GRU
    }

}


