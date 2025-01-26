#[macro_export]
macro_rules! node_cell {
    ($name:ident, $arity:expr) => {
        impl NodeCell for $name {
            // fn arity(&self) -> Arity {
            //     $arity
            // }

            // fn new_instance(&self) -> Self {
            //     self.clone()
            // }
        }
    };
}
