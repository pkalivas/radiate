use std::ops::Range;

use super::PyCodec;
use crate::{PyAnyObject, PyChromosome, PyGene, PyGenotype};
use num_traits::NumCast;
use numpy::Element;
use pyo3::{Bound, IntoPyObject, IntoPyObjectExt, PyAny, PyResult};
use radiate::{
    Chromosome, Codec, FloatChromosome, Gene, Genotype, IntChromosome, chromosomes::NumericAllele,
};
use radiate_utils::DataType;

pub trait CodecBuilder<C: Chromosome, T> {
    fn build(self) -> PyCodec<C, T>;
}

#[derive(Clone)]
pub enum TypedNumericCodec {
    U8(PyCodec<IntChromosome<u8>, PyAnyObject>),
    U16(PyCodec<IntChromosome<u16>, PyAnyObject>),
    U32(PyCodec<IntChromosome<u32>, PyAnyObject>),
    U64(PyCodec<IntChromosome<u64>, PyAnyObject>),
    U128(PyCodec<IntChromosome<u128>, PyAnyObject>),
    I8(PyCodec<IntChromosome<i8>, PyAnyObject>),
    I16(PyCodec<IntChromosome<i16>, PyAnyObject>),
    I32(PyCodec<IntChromosome<i32>, PyAnyObject>),
    I64(PyCodec<IntChromosome<i64>, PyAnyObject>),
    I128(PyCodec<IntChromosome<i128>, PyAnyObject>),
    F32(PyCodec<FloatChromosome<f32>, PyAnyObject>),
    F64(PyCodec<FloatChromosome<f64>, PyAnyObject>),
}

impl TypedNumericCodec {
    pub fn encode(&self) -> PyGenotype {
        match self {
            TypedNumericCodec::U8(codec) => codec.encode().into(),
            TypedNumericCodec::U16(codec) => codec.encode().into(),
            TypedNumericCodec::U32(codec) => codec.encode().into(),
            TypedNumericCodec::U64(codec) => codec.encode().into(),
            TypedNumericCodec::U128(codec) => codec.encode().into(),
            TypedNumericCodec::I8(codec) => codec.encode().into(),
            TypedNumericCodec::I16(codec) => codec.encode().into(),
            TypedNumericCodec::I32(codec) => codec.encode().into(),
            TypedNumericCodec::I64(codec) => codec.encode().into(),
            TypedNumericCodec::I128(codec) => codec.encode().into(),
            TypedNumericCodec::F32(codec) => codec.encode().into(),
            TypedNumericCodec::F64(codec) => codec.encode().into(),
        }
    }

    pub fn decode_with_py<'py>(
        &self,
        py: pyo3::Python<'py>,
        genotype: &PyGenotype,
    ) -> PyResult<Bound<'py, PyAny>> {
        match self {
            TypedNumericCodec::U8(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::U16(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::U32(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::U64(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::U128(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::I8(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::I16(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::I32(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::I64(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::I128(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::F32(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
            TypedNumericCodec::F64(codec) => codec
                .decode_with_py(py, &genotype.clone().into())
                .into_bound_py_any(py),
        }
    }

    pub fn dtype(&self) -> DataType {
        match self {
            TypedNumericCodec::U8(_) => DataType::UInt8,
            TypedNumericCodec::U16(_) => DataType::UInt16,
            TypedNumericCodec::U32(_) => DataType::UInt32,
            TypedNumericCodec::U64(_) => DataType::UInt64,
            TypedNumericCodec::U128(_) => DataType::UInt128,
            TypedNumericCodec::I8(_) => DataType::Int8,
            TypedNumericCodec::I16(_) => DataType::Int16,
            TypedNumericCodec::I32(_) => DataType::Int32,
            TypedNumericCodec::I64(_) => DataType::Int64,
            TypedNumericCodec::I128(_) => DataType::Int128,
            TypedNumericCodec::F32(_) => DataType::Float32,
            TypedNumericCodec::F64(_) => DataType::Float64,
        }
    }
}

pub struct NumericCodecBuilder<T> {
    pub shape: Vec<usize>,
    pub init_range: Option<(T, T)>,
    pub bound_range: Option<(T, T)>,
    pub genes: Option<Vec<PyGene>>,
    pub chromosomes: Option<Vec<PyChromosome>>,
    pub use_numpy: bool,
    pub dtype: DataType,
}

impl<T> NumericCodecBuilder<T> {
    pub fn shape(mut self, shape: Vec<usize>) -> Self {
        self.shape = shape;
        self
    }

    pub fn init_range(mut self, range: Option<(T, T)>) -> Self {
        if range.is_some() {
            self.init_range = range;
        }

        self
    }

    pub fn bound_range(mut self, range: Option<(T, T)>) -> Self {
        if range.is_some() {
            self.bound_range = range;
        }

        self
    }

    pub fn genes(mut self, genes: Vec<PyGene>) -> Self {
        self.genes = Some(genes);
        self
    }

    pub fn chromosomes(mut self, chromosomes: Vec<PyChromosome>) -> Self {
        self.chromosomes = Some(chromosomes);
        self
    }

    pub fn use_numpy(mut self, use_numpy: bool) -> Self {
        self.use_numpy = use_numpy;
        self
    }

    pub fn dtype(mut self, dtype: DataType) -> Self {
        self.dtype = dtype;
        self
    }

    pub fn materialize_genes<G, C>(genes: &[PyGene]) -> C
    where
        C: Chromosome<Gene = G> + From<Vec<G>>,
        G: Gene + From<PyGene>,
    {
        C::from(
            genes
                .iter()
                .map(|gene| G::from(gene.clone()))
                .collect::<Vec<G>>(),
        )
    }

    pub fn materialize_chromosomes<G, C>(chromosomes: &[PyChromosome]) -> Vec<C>
    where
        C: Chromosome<Gene = G> + From<Vec<G>>,
        G: Gene + From<PyGene>,
    {
        chromosomes
            .iter()
            .map(|chrom| Self::materialize_genes(&chrom.genes))
            .collect::<Vec<C>>()
    }
}

impl Default for NumericCodecBuilder<i64> {
    fn default() -> Self {
        NumericCodecBuilder {
            shape: vec![1],
            init_range: Some((0, 10)),
            bound_range: None,
            genes: None,
            chromosomes: None,
            use_numpy: false,
            dtype: DataType::Int64,
        }
    }
}

impl Default for NumericCodecBuilder<f64> {
    fn default() -> Self {
        NumericCodecBuilder {
            shape: vec![1],
            init_range: Some((0.0, 1.0)),
            bound_range: None,
            genes: None,
            chromosomes: None,
            use_numpy: false,
            dtype: DataType::Float64,
        }
    }
}

impl<I, A, G, C> CodecBuilder<C, PyAnyObject> for NumericCodecBuilder<I>
where
    I: NumCast,
    A: NumericAllele
        + Element
        + Copy
        + Default
        + NumCast
        + for<'py> IntoPyObject<'py>
        + for<'py> IntoPyObjectExt<'py>
        + 'static,
    G: Gene<Allele = A> + From<PyGene>,
    C: Chromosome<Gene = G> + Clone + From<Vec<G>> + From<(usize, Range<A>, Range<A>)> + 'static,
{
    fn build(self) -> PyCodec<C, PyAnyObject> {
        let val_range = self
            .init_range
            .and_then(|(min, max)| A::from(min).zip(A::from(max)).map(|(min, max)| min..max))
            .unwrap_or_else(|| {
                self.dtype
                    .min()
                    .zip(self.dtype.max())
                    .and_then(|(min, max)| {
                        min.value()
                            .clone()
                            .extract::<A>()
                            .zip(max.value().clone().extract::<A>())
                            .map(|(min, max)| min..max)
                    })
                    .unwrap()
            });

        let bound_range = self
            .bound_range
            .and_then(|rng| {
                A::from(rng.0)
                    .zip(A::from(rng.1))
                    .map(|(min, max)| min..max)
            })
            .unwrap_or(val_range.clone());
        let lengths = self.shape.clone();
        let cloned_lengths = lengths.clone();
        let use_numpy = self.use_numpy;

        // Capture write-relevant info for the snapshot closure.
        let dtype_label = format!("{:?}", self.dtype);
        let codec_kind = match self.dtype {
            DataType::Float32 | DataType::Float64 => "FloatCodec",
            _ => "IntCodec",
        };
        let shape_for_write = lengths.clone();
        let val_start: f64 = num_traits::cast(val_range.start).unwrap_or(f64::NAN);
        let val_end: f64 = num_traits::cast(val_range.end).unwrap_or(f64::NAN);
        let bound_start: f64 = num_traits::cast(bound_range.start).unwrap_or(f64::NAN);
        let bound_end: f64 = num_traits::cast(bound_range.end).unwrap_or(f64::NAN);
        let gene_count = self.genes.as_ref().map(|g| g.len());
        let chrom_count = self.chromosomes.as_ref().map(|c| c.len());

        let codec = if let Some(genes) = &self.genes {
            let materialized_chromosome = Self::materialize_genes::<G, C>(genes);

            PyCodec::new()
                .with_encoder(move || Genotype::from(materialized_chromosome.clone()))
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                })
        } else if let Some(chromosomes) = &self.chromosomes {
            let materialized_chromosomes = Self::materialize_chromosomes::<G, C>(chromosomes);

            PyCodec::new()
                .with_encoder(move || Genotype::from(materialized_chromosomes.clone()))
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                })
        } else {
            PyCodec::new()
                .with_encoder(move || {
                    Genotype::from(
                        cloned_lengths
                            .iter()
                            .map(|len| C::from((*len, val_range.clone(), bound_range.clone())))
                            .collect::<Vec<C>>(),
                    )
                })
                .with_decoder(move |py, geno| PyAnyObject {
                    inner: super::decode_genotype_to_array(py, geno, use_numpy)
                        .unwrap()
                        .unbind()
                        .into_any(),
                })
        };

        codec.with_write(move |w| {
            writeln!(w, "type: {}", codec_kind)?;
            writeln!(w, "dtype: {}", dtype_label)?;
            writeln!(w, "shape: {:?}", shape_for_write)?;
            writeln!(w, "value_range: [{}, {}]", val_start, val_end)?;
            writeln!(w, "bound_range: [{}, {}]", bound_start, bound_end)?;
            writeln!(w, "use_numpy: {}", use_numpy)?;
            if let Some(n) = gene_count {
                writeln!(w, "genes: {}", n)?;
            }
            if let Some(n) = chrom_count {
                writeln!(w, "chromosomes: {}", n)?;
            }
            Ok(())
        })
    }
}
