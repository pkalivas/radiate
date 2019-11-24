pub mod neurons;
pub mod neat;
pub mod edge;
pub mod neatenv;
pub mod layer;
pub mod activation;


/// keep track of innovation numbers for neat 
/// this thing doesn't deserve it's own file its too small
pub mod counter {
        
    #[derive(Debug, Clone)]
    pub struct Counter {
        num: i32
    }

    impl Counter {
        pub fn new() -> Self {
            Counter {
                num: 0
            }
        }

        pub fn next(&mut self) -> i32 {
            let result = self.num;
            self.num += 1;
            result
        }

        pub fn roll_back(&mut self, num: i32) {
            self.num -= num;
        }
    }
}