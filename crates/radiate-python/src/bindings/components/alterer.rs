use crate::{ObjectValue, PyGeneType, PyPopulation, conversion::Wrap, gene::PyChromosomeType};
use pyo3::{
    Bound, FromPyObject, IntoPyObjectExt, PyAny, PyErr, PyResult, Python,
    exceptions::PyValueError,
    pyclass, pymethods,
    types::{PyAnyMethods, PyDict, PyDictMethods, PyString},
};
use radiate::{
    Alter, ArithmeticMutator, BitChromosome, BlendCrossover, CharChromosome, Chromosome, Crossover,
    FloatChromosome, GaussianMutator, IntChromosome, IntermediateCrossover, MeanCrossover,
    MultiPointCrossover, Mutate, Population, ScrambleMutator, ShuffleCrossover,
    SimulatedBinaryCrossover, SwapMutator, UniformCrossover, UniformMutator, alters,
};
use std::{hash::Hash, vec};

#[pyclass(unsendable)]
#[derive(Clone, Debug)]
pub struct PyAlterer {
    name: String,
    args: ObjectValue,
    allowed_genes: Vec<PyGeneType>,
    chromosomes: Vec<PyChromosomeType>,
}

#[pymethods]
impl PyAlterer {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn __str__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.__repr__(py)
    }

    pub fn __repr__<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let repr = format!(
            "Alterer(name={}, args={})",
            self.name,
            self.args.inner.bind(py)
        );

        PyString::new(py, &repr).into_bound_py_any(py)
    }

    pub fn __eq__(&self, other: &Self) -> bool {
        let mut state = std::hash::DefaultHasher::new();
        self.name == other.name && self.args.hash(&mut state) == other.args.hash(&mut state)
    }

    pub fn args<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        self.args.inner.bind(py).into_bound_py_any(py)
    }

    pub fn allowed_genes(&self) -> PyResult<Vec<PyGeneType>> {
        Ok(self.allowed_genes.clone())
    }

    pub fn is_valid_for_gene(&self, gene_type: &str) -> bool {
        self.allowed_genes.iter().any(|g| g.name() == gene_type)
    }

    pub fn alter<'py>(&self, py: Python<'py>, population: PyPopulation) -> PyResult<PyPopulation> {
        if population.gene_type() == String::from("FloatGene") {
            return alter_population::<FloatChromosome>(py, vec![self.clone()], population);
        } else if population.gene_type() == String::from("IntGene") {
            return alter_population::<IntChromosome<i32>>(py, vec![self.clone()], population);
        } else if population.gene_type() == String::from("BitGene") {
            return alter_population::<BitChromosome>(py, vec![self.clone()], population);
        } else if population.gene_type() == String::from("CharGene") {
            return alter_population::<CharChromosome>(py, vec![self.clone()], population);
        } else {
            return Err(PyValueError::new_err(format!(
                "Unsupported gene type: {}",
                population.gene_type()
            )));
        }
    }
}

#[pymethods]
impl PyAlterer {
    #[staticmethod]
    #[pyo3(signature = (rate=None, alpha=None))]
    pub fn blend_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        alpha: Option<f32>,
    ) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(a) = alpha {
            args.set_item("alpha", a)?;
        }

        Ok(PyAlterer {
            name: "blend_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
            chromosomes: vec![PyChromosomeType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None, alpha=None))]
    pub fn intermediate_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        alpha: Option<f32>,
    ) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(a) = alpha {
            args.set_item("alpha", a)?;
        }

        Ok(PyAlterer {
            name: "intermediate_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
            chromosomes: vec![PyChromosomeType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn mean_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "mean_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float, PyGeneType::Int],
            chromosomes: vec![PyChromosomeType::Float, PyChromosomeType::Int],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None, num_points=None))]
    pub fn multi_point_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        num_points: Option<usize>,
    ) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(n) = num_points {
            args.set_item("num_points", n)?;
        }
        if let Some(r) = rate {
            if !(0.0..=1.0).contains(&r) {
                return Err(PyValueError::new_err("Rate must be between 0 and 1"));
            }
        }

        if let Some(n) = num_points {
            if !(1..=usize::MAX).contains(&n) {
                return Err(PyValueError::new_err(
                    "Num points must be a positive integer",
                ));
            }
        }

        Ok(PyAlterer {
            name: "multi_point_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosomes: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn shuffle_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "shuffle_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Bit,
                PyGeneType::Char,
                PyGeneType::Float,
                PyGeneType::Int,
            ],
            chromosomes: vec![
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
                PyChromosomeType::Float,
                PyChromosomeType::Int,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None, contiguty=None))]
    pub fn simulated_binary_crossover<'py>(
        py: Python<'py>,
        rate: Option<f32>,
        contiguty: Option<f32>,
    ) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }
        if let Some(c) = contiguty {
            args.set_item("contiguty", c)?;
        }

        Ok(PyAlterer {
            name: "simulated_binary_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
            chromosomes: vec![PyChromosomeType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn uniform_crossover<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "uniform_crossover".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosomes: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn arithmetic_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "arithmetic_mutator".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float, PyGeneType::Int],
            chromosomes: vec![PyChromosomeType::Float, PyChromosomeType::Int],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn gaussian_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "gaussian_mutator".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Float],
            chromosomes: vec![PyChromosomeType::Float],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn scramble_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "scramble_mutator".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![PyGeneType::Bit, PyGeneType::Char],
            chromosomes: vec![PyChromosomeType::Bit, PyChromosomeType::Char],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn swap_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "swap_mutator".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Bit,
                PyGeneType::Char,
                PyGeneType::Float,
                PyGeneType::Int,
            ],
            chromosomes: vec![
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
                PyChromosomeType::Float,
                PyChromosomeType::Int,
            ],
        })
    }

    #[staticmethod]
    #[pyo3(signature = (rate=None))]
    pub fn uniform_mutator<'py>(py: Python<'py>, rate: Option<f32>) -> PyResult<PyAlterer> {
        let args = PyDict::new(py);
        if let Some(r) = rate {
            args.set_item("rate", r)?;
        }

        Ok(PyAlterer {
            name: "uniform_mutator".into(),
            args: ObjectValue { inner: args.into() },
            allowed_genes: vec![
                PyGeneType::Float,
                PyGeneType::Int,
                PyGeneType::Bit,
                PyGeneType::Char,
            ],
            chromosomes: vec![
                PyChromosomeType::Float,
                PyChromosomeType::Int,
                PyChromosomeType::Bit,
                PyChromosomeType::Char,
            ],
        })
    }
}

fn alter_population<C>(
    py: Python<'_>,
    alters: Vec<PyAlterer>,
    population: PyPopulation,
) -> PyResult<PyPopulation>
where
    C: Chromosome + 'static,
    Population<C>: From<PyPopulation>,
    PyPopulation: From<Population<C>>,
{
    for alter in alters.iter() {
        if !alter.is_valid_for_gene(population.gene_type().as_str()) {
            return Err(PyValueError::new_err(format!(
                "Alterer '{}' does not support gene type '{}'",
                alter.name(),
                population.gene_type()
            )));
        }
    }
    let alter2 = alters
        .into_bound_py_any(py)?
        .extract::<Wrap<Vec<Box<dyn Alter<C>>>>>()?
        .0;

    if alter2.is_empty() {
        return Err(PyValueError::new_err(
            "At least one alterer must be specified",
        ));
    }

    let mut pop = Population::<C>::from(population);

    for alter in alter2 {
        alter.alter(&mut pop, 0);
    }

    Ok(PyPopulation::from(pop))
}

impl<'py> FromPyObject<'py> for Wrap<BlendCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if alter.name() != "blend_crossover" {
            return Err(PyValueError::new_err(
                "BlendCrossover must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;
        let alpha = alter.args(ob.py())?.get_item("alpha")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }
        if !(0.0..=1.0).contains(&alpha) {
            return Err(PyValueError::new_err("Alpha must be between 0 and 1"));
        }

        Ok(Wrap(BlendCrossover::new(rate, alpha)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<IntermediateCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if alter.name() != "intermediate_crossover" {
            return Err(PyValueError::new_err(
                "IntermediateCrossover must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;
        let alpha = alter.args(ob.py())?.get_item("alpha")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }
        if !(0.0..=1.0).contains(&alpha) {
            return Err(PyValueError::new_err("Alpha must be between 0 and 1"));
        }

        Ok(Wrap(IntermediateCrossover::new(rate, alpha)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<MeanCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if alter.name() != "mean_crossover" {
            return Err(PyValueError::new_err(
                "MeanCrossover must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(MeanCrossover::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<MultiPointCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "multi_point_crossover" {
            return Err(PyValueError::new_err(
                "MultiPointCrossover must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;
        let num_points = alter
            .args(ob.py())?
            .get_item("num_points")?
            .extract::<usize>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        if !(1..=usize::MAX).contains(&num_points) {
            return Err(PyValueError::new_err(
                "Num points must be a positive integer",
            ));
        }

        Ok(Wrap(MultiPointCrossover::new(rate, num_points)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<ShuffleCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "shuffle_crossover" {
            return Err(PyValueError::new_err(
                "ShuffleCrossover must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(ShuffleCrossover::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<SimulatedBinaryCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if !alter.allowed_genes.contains(&PyGeneType::Float) {
            return Err(PyValueError::new_err(
                "SimulatedBinaryCrossover requires Float genes",
            ));
        }

        if alter.name() != "simulated_binary_crossover" {
            return Err(PyValueError::new_err(
                "SimulatedBinaryCrossover must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;
        let contiguity = alter
            .args(ob.py())?
            .get_item("contiguty")?
            .extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }
        if !(0.0..=1.0).contains(&contiguity) {
            return Err(PyValueError::new_err("Contiguity must be between 0 and 1"));
        }

        Ok(Wrap(SimulatedBinaryCrossover::new(rate, contiguity)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<UniformCrossover> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "uniform_crossover" {
            return Err(PyValueError::new_err(
                "UniformCrossover must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(UniformCrossover::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<ArithmeticMutator> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "arithmetic_mutator" {
            return Err(PyValueError::new_err(
                "ArithmeticMutator must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(ArithmeticMutator::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<GaussianMutator> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "gaussian_mutator" {
            return Err(PyValueError::new_err(
                "GaussianMutator must be created using the static method",
            ));
        }

        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(GaussianMutator::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<ScrambleMutator> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;
        if alter.name() != "scramble_mutator" {
            return Err(PyValueError::new_err(
                "ScrambleMutator must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(ScrambleMutator::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<SwapMutator> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if alter.name() != "swap_mutator" {
            return Err(PyValueError::new_err(
                "SwapMutator must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(SwapMutator::new(rate)))
    }
}

impl<'py> FromPyObject<'py> for Wrap<UniformMutator> {
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alter = ob.extract::<PyAlterer>()?;

        if alter.name() != "uniform_mutator" {
            return Err(PyValueError::new_err(
                "UniformMutator must be created using the static method",
            ));
        }
        let rate = alter.args(ob.py())?.get_item("rate")?.extract::<f32>()?;

        if !(0.0..=1.0).contains(&rate) {
            return Err(PyValueError::new_err("Rate must be between 0 and 1"));
        }

        Ok(Wrap(UniformMutator::new(rate)))
    }
}

impl<'py, C> FromPyObject<'py> for Wrap<Vec<Box<dyn Alter<C>>>>
where
    C: Chromosome + 'static,
{
    fn extract_bound(ob: &Bound<'py, PyAny>) -> PyResult<Self> {
        let alters = ob.extract::<Vec<PyAlterer>>()?;
        if alters.is_empty() {
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                "At least one alterer must be specified",
            ));
        }

        let chromosome_name = std::any::type_name::<C>()
            .split("::")
            .last()
            .map(|s| s.split('<').next())
            .flatten()
            .unwrap_or_default();

        let typed_params = alters
            .into_iter()
            .filter(|alt| alt.chromosomes.iter().any(|c| c.name() == chromosome_name))
            .collect::<Vec<PyAlterer>>();

        match chromosome_name {
            "FloatChromosome" => {
                let alters = typed_params
                    .into_iter()
                    .map(|alter| alter.into_bound_py_any(ob.py()))
                    .filter_map(|alter| alter.ok())
                    .map(|alter| {
                        if let Ok(blend) = alter.extract::<Wrap<BlendCrossover>>() {
                            return alters!(blend.0);
                        } else if let Ok(intermediate) =
                            alter.extract::<Wrap<IntermediateCrossover>>()
                        {
                            return alters!(intermediate.0);
                        } else if let Ok(mean) = alter.extract::<Wrap<MeanCrossover>>() {
                            return alters!(mean.0);
                        } else if let Ok(multi_point) = alter.extract::<Wrap<MultiPointCrossover>>()
                        {
                            return alters!(multi_point.0);
                        } else if let Ok(shuffle) = alter.extract::<Wrap<ShuffleCrossover>>() {
                            return alters!(shuffle.0);
                        } else if let Ok(simulated_binary) =
                            alter.extract::<Wrap<SimulatedBinaryCrossover>>()
                        {
                            return alters!(simulated_binary.0);
                        } else if let Ok(uniform) = alter.extract::<Wrap<UniformCrossover>>() {
                            return alters!(uniform.0);
                        } else if let Ok(arithmetic) = alter.extract::<Wrap<ArithmeticMutator>>() {
                            return alters!(arithmetic.0);
                        } else if let Ok(gaussian) = alter.extract::<Wrap<GaussianMutator>>() {
                            return alters!(gaussian.0);
                        } else if let Ok(scramble) = alter.extract::<Wrap<ScrambleMutator>>() {
                            return alters!(scramble.0);
                        } else if let Ok(swap) = alter.extract::<Wrap<SwapMutator>>() {
                            return alters!(swap.0);
                        } else if let Ok(uniform_mutator) = alter.extract::<Wrap<UniformMutator>>()
                        {
                            return alters!(uniform_mutator.0);
                        } else {
                            return Vec::<Box<dyn Alter<FloatChromosome>>>::new();
                        }
                    })
                    .flat_map(|alters| alters.into_iter())
                    .collect::<Vec<_>>();

                return Ok(Wrap(unsafe {
                    std::mem::transmute::<
                        Vec<Box<dyn Alter<FloatChromosome>>>,
                        Vec<Box<dyn Alter<C>>>,
                    >(alters)
                }));
            }
            "IntChromosome" => {
                let alters = typed_params
                    .into_iter()
                    .map(|alter| alter.into_bound_py_any(ob.py()))
                    .filter_map(|alter| alter.ok())
                    .map(|alter| {
                        if let Ok(mean) = alter.extract::<Wrap<MeanCrossover>>() {
                            return alters!(mean.0);
                        } else if let Ok(multi_point) = alter.extract::<Wrap<MultiPointCrossover>>()
                        {
                            return alters!(multi_point.0);
                        } else if let Ok(shuffle) = alter.extract::<Wrap<ShuffleCrossover>>() {
                            return alters!(shuffle.0);
                        } else if let Ok(uniform) = alter.extract::<Wrap<UniformCrossover>>() {
                            return alters!(uniform.0);
                        } else if let Ok(arithmetic) = alter.extract::<Wrap<ArithmeticMutator>>() {
                            return alters!(arithmetic.0);
                        } else if let Ok(scramble) = alter.extract::<Wrap<ScrambleMutator>>() {
                            return alters!(scramble.0);
                        } else if let Ok(swap) = alter.extract::<Wrap<SwapMutator>>() {
                            return alters!(swap.0);
                        } else if let Ok(uniform_mutator) = alter.extract::<Wrap<UniformMutator>>()
                        {
                            return alters!(uniform_mutator.0);
                        } else {
                            return Vec::<Box<dyn Alter<IntChromosome<i32>>>>::new();
                        }
                    })
                    .flat_map(|alters| alters.into_iter())
                    .collect::<Vec<_>>();

                return Ok(Wrap(unsafe {
                    std::mem::transmute::<
                        Vec<Box<dyn Alter<IntChromosome<i32>>>>,
                        Vec<Box<dyn Alter<C>>>,
                    >(alters)
                }));
            }
            "CharChromosome" => {
                let alters = typed_params
                    .into_iter()
                    .map(|alter| alter.into_bound_py_any(ob.py()))
                    .filter_map(|alter| alter.ok())
                    .map(|alter| {
                        if let Ok(multi_point) = alter.extract::<Wrap<MultiPointCrossover>>() {
                            return alters!(multi_point.0);
                        } else if let Ok(shuffle) = alter.extract::<Wrap<ShuffleCrossover>>() {
                            return alters!(shuffle.0);
                        } else if let Ok(uniform) = alter.extract::<Wrap<UniformCrossover>>() {
                            return alters!(uniform.0);
                        } else if let Ok(scramble) = alter.extract::<Wrap<ScrambleMutator>>() {
                            return alters!(scramble.0);
                        } else if let Ok(swap) = alter.extract::<Wrap<SwapMutator>>() {
                            return alters!(swap.0);
                        } else if let Ok(uniform_mutator) = alter.extract::<Wrap<UniformMutator>>()
                        {
                            return alters!(uniform_mutator.0);
                        } else {
                            return Vec::<Box<dyn Alter<CharChromosome>>>::new();
                        }
                    })
                    .flat_map(|alters| alters.into_iter())
                    .collect::<Vec<_>>();

                return Ok(Wrap(unsafe {
                    std::mem::transmute::<Vec<Box<dyn Alter<CharChromosome>>>, Vec<Box<dyn Alter<C>>>>(
                        alters,
                    )
                }));
            }
            "BitChromosome" => {
                let alters = typed_params
                    .into_iter()
                    .map(|alter| alter.into_bound_py_any(ob.py()))
                    .filter_map(|alter| alter.ok())
                    .map(|alter| {
                        if let Ok(multi_point) = alter.extract::<Wrap<MultiPointCrossover>>() {
                            return alters!(multi_point.0);
                        } else if let Ok(shuffle) = alter.extract::<Wrap<ShuffleCrossover>>() {
                            return alters!(shuffle.0);
                        } else if let Ok(uniform) = alter.extract::<Wrap<UniformCrossover>>() {
                            return alters!(uniform.0);
                        } else if let Ok(scramble) = alter.extract::<Wrap<ScrambleMutator>>() {
                            return alters!(scramble.0);
                        } else if let Ok(swap) = alter.extract::<Wrap<SwapMutator>>() {
                            return alters!(swap.0);
                        } else if let Ok(uniform_mutator) = alter.extract::<Wrap<UniformMutator>>()
                        {
                            return alters!(uniform_mutator.0);
                        } else {
                            return Vec::<Box<dyn Alter<BitChromosome>>>::new();
                        }
                    })
                    .flat_map(|alters| alters.into_iter())
                    .collect::<Vec<_>>();

                return Ok(Wrap(unsafe {
                    std::mem::transmute::<Vec<Box<dyn Alter<BitChromosome>>>, Vec<Box<dyn Alter<C>>>>(
                        alters,
                    )
                }));
            }
            _ => {
                return Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(
                    "Unsupported Chromosome type",
                ));
            }
        }
    }
}
