pub mod layer;
pub mod dense;
pub mod lstm;


pub mod layertype {

    #[derive(Debug, PartialEq, Clone, Copy)]
    pub enum LayerType {
        DensePool,
        Dense,
        LSTM, 
    }
}



pub mod vectorops {

        /// multiply two vectors element wise
        #[inline]
        pub fn element_multiply(one: &mut Vec<f64>, two: &Vec<f64>) {
            one.iter_mut()
                .zip(two.iter())
                .for_each(|(a, b)| {
                    *a *= b
                });
        }
    
    
    
        /// invert a vector that is already holding values between 0 and 1
        #[inline]
        pub fn element_invert(one: &mut Vec<f64>) {
            one.iter_mut()
                .for_each(|a| *a = 1.0 - *a);
        }
    
    
    
        /// add elements from vectors together element wise
        #[inline]
        pub fn element_add(one: &mut Vec<f64>, two: &Vec<f64>) {
            one.iter_mut()
                .zip(two.iter())
                .for_each(|(a, b)| {
                    *a += b
                });
        }

}
