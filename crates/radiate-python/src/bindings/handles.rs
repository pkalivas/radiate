use crate::PyAnyObject;
use pyo3::PyResult;
use radiate::{
    BitChromosome, CharChromosome, FloatChromosome, Generation, GeneticEngine,
    GeneticEngineBuilder, Graph, GraphChromosome, IntChromosome, Limit, Op, PermutationChromosome,
    Tree, TreeChromosome,
};
use radiate_error::radiate_py_err;
use serde::{Deserialize, Serialize};

type GenIter<C, T> = Box<dyn Iterator<Item = Generation<C, T>>>;

/// Single source of truth for every gene-type variant the Python bindings support.
/// `EngineBuilderHandle`, `EngineHandle`, `StepHandle`, `EpochHandle`, and every
/// transform between them are generated from this list — adding a gene type means
/// adding one row here instead of touching four parallel enums by hand.
macro_rules! gene_variants {
    ($mac:path) => {
        $mac! {
            UInt8       => IntChromosome<u8>,            PyAnyObject;
            UInt16      => IntChromosome<u16>,           PyAnyObject;
            UInt32      => IntChromosome<u32>,           PyAnyObject;
            UInt64      => IntChromosome<u64>,           PyAnyObject;
            Int8        => IntChromosome<i8>,            PyAnyObject;
            Int16       => IntChromosome<i16>,           PyAnyObject;
            Int32       => IntChromosome<i32>,           PyAnyObject;
            Int64       => IntChromosome<i64>,           PyAnyObject;
            Float32     => FloatChromosome<f32>,         PyAnyObject;
            Float64     => FloatChromosome<f64>,         PyAnyObject;
            Char        => CharChromosome,               PyAnyObject;
            Bit         => BitChromosome,                PyAnyObject;
            Permutation => PermutationChromosome<usize>, PyAnyObject;
            Graph       => GraphChromosome<Op<f32>>,     Graph<Op<f32>>;
            Tree        => TreeChromosome<Op<f32>>,      Vec<Tree<Op<f32>>>;
        }
    };
}

#[macro_export]
macro_rules! match_variant {
    ($enum_path:path, $handle:expr, $bind:ident => $body:expr) => {{
        use $enum_path::*;
        match $handle {
            UInt8($bind) => $body,
            UInt16($bind) => $body,
            UInt32($bind) => $body,
            UInt64($bind) => $body,
            Int8($bind) => $body,
            Int16($bind) => $body,
            Int32($bind) => $body,
            Int64($bind) => $body,
            Float32($bind) => $body,
            Float64($bind) => $body,
            Char($bind) => $body,
            Bit($bind) => $body,
            Permutation($bind) => $body,
            Graph($bind) => $body,
            Tree($bind) => $body,
        }
    }};
}

macro_rules! define_builder_enum {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        pub enum EngineBuilderHandle {
            Empty,
            $( $variant(GeneticEngineBuilder<$chrom, $decoded>), )*
        }
    };
}
gene_variants!(define_builder_enum);

macro_rules! define_engine_enum {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        pub enum EngineHandle {
            $( $variant(GeneticEngine<$chrom, $decoded>), )*
        }
    };
}
gene_variants!(define_engine_enum);

macro_rules! define_step_enum {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        pub enum EngineIterHandle{
            $( $variant(GenIter<$chrom, $decoded>), )*
        }

        unsafe impl Send for EngineIterHandle {}
    };
}
gene_variants!(define_step_enum);

macro_rules! define_epoch_enum {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        #[derive(Clone, Serialize, Deserialize)]
        pub enum EpochHandle {
            $( $variant(Generation<$chrom, $decoded>), )*
        }

        $(
            impl From<Generation<$chrom, $decoded>> for EpochHandle {
                fn from(epoch: Generation<$chrom, $decoded>) -> Self {
                    EpochHandle::$variant(epoch)
                }
            }

            impl From<EpochHandle> for Generation<$chrom, $decoded> {
                fn from(handle: EpochHandle) -> Self {
                    match handle {
                        EpochHandle::$variant(epoch) => epoch,
                        _ => panic!("Invalid epoch handle variant for this type"),
                    }
                }
            }

        )*
    };
}
gene_variants!(define_epoch_enum);

macro_rules! define_try_build {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        impl EngineBuilderHandle {
            pub fn try_build(self) -> PyResult<EngineHandle> {
                match self {
                    EngineBuilderHandle::Empty => {
                        Err(radiate_py_err!("Cannot build an Empty builder"))
                    }
                    $( EngineBuilderHandle::$variant(b) => {
                        Ok(EngineHandle::$variant(b.try_build()?))
                    } )*
                }
            }
        }
    };
}
gene_variants!(define_try_build);

macro_rules! define_into_step {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        impl EngineHandle {
            pub fn into_iter_handle(self, limits: Vec<Limit>) -> EngineIterHandle {
                match self {
                    $( EngineHandle::$variant(eng) => {
                        EngineIterHandle::$variant(Box::new(eng.iter().limit(limits)))
                    } )*
                }
            }
        }
    };
}
gene_variants!(define_into_step);

macro_rules! define_step_next {
    ($($variant:ident => $chrom:ty, $decoded:ty);* $(;)?) => {
        impl EngineIterHandle {
            pub fn next_epoch(&mut self) -> Option<EpochHandle> {
                match self {
                    $( EngineIterHandle::$variant(it) => it.next().map(EpochHandle::$variant), )*
                }
            }
        }
    };
}
gene_variants!(define_step_next);
