pub mod layer;
pub mod dense;
pub mod lstm;
pub mod gru;
pub mod vectorops;


pub mod layertype {

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum LayerType {
        DensePool,
        Dense,
        LSTM, 
        GRU
    }
}


