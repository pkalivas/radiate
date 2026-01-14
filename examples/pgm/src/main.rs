use radiate::*;

pub fn sample_categorical(probs: &[f32]) -> usize {
    // probs must sum to 1
    let r = random_provider::range(0.0..1.0);
    let mut acc = 0.0f32;
    for (i, &p) in probs.iter().enumerate() {
        acc += p;
        if r <= acc {
            return i;
        }
    }
    probs.len().saturating_sub(1)
}

/// rows: Vec<Vec<Option<usize>>> (fully observed: Some(state) everywhere)
pub fn make_iid_dataset(n: usize, dists: &[Vec<f32>]) -> Vec<Vec<Option<usize>>> {
    (0..n)
        .map(|_| dists.iter().map(|p| Some(sample_categorical(p))).collect())
        .collect()
}

pub fn chain_model_abc() -> Result<Vec<DiscreteFactor>, String> {
    // A(2) -> B(2) -> C(2)
    let a = VarSpec::new(0, 2);
    let b = VarSpec::new(1, 2);
    let c = VarSpec::new(2, 2);

    // P(A): logits [0, 1]
    let mut p_a = DiscreteFactor::new(vec![a], vec![0.0, 1.0])?;
    p_a.normalize_rows(VarId(0))?;

    // P(B|A): scope [A,B] (child is B)
    let mut p_ba = DiscreteFactor::new(
        vec![a, b],
        vec![
            2.0, 0.0, // A=0 -> B=0..1
            0.0, 2.0, // A=1 -> B=0..1
        ],
    )?;
    p_ba.normalize_rows(VarId(1))?;

    // P(C|B): scope [B,C] (child is C)
    let mut p_cb = DiscreteFactor::new(
        vec![b, c],
        vec![
            2.0, 0.0, // B=0 -> C=0..1
            0.0, 2.0, // B=1 -> C=0..1
        ],
    )?;
    p_cb.normalize_rows(VarId(2))?;

    Ok(vec![p_a, p_ba, p_cb])
}

fn sample_from_log_probs(logps: &[f32]) -> usize {
    // logps already represent log(prob)
    let max = logps.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let mut sum = 0.0f32;
    let mut probs = vec![0.0; logps.len()];
    for (i, &lp) in logps.iter().enumerate() {
        let p = (lp - max).exp();
        probs[i] = p;
        sum += p;
    }
    let r = random_provider::range(0.0..sum);
    let mut acc = 0.0;
    for (i, p) in probs.iter().enumerate() {
        acc += *p;
        if r <= acc {
            return i;
        }
    }
    probs.len() - 1
}

pub fn sample_chain_dataset(n: usize) -> Result<Vec<Vec<Option<usize>>>, String> {
    let factors = chain_model_abc()?;
    let p_a = &factors[0];
    let p_ba = &factors[1];
    let p_cb = &factors[2];

    let mut rows = Vec::with_capacity(n);
    for _ in 0..n {
        // A
        let a = sample_from_log_probs(&[p_a.log_value_aligned(&[0]), p_a.log_value_aligned(&[1])]);

        // B | A
        let b0 = p_ba.log_value_aligned(&[a, 0]);
        let b1 = p_ba.log_value_aligned(&[a, 1]);
        let b = sample_from_log_probs(&[b0, b1]);

        // C | B
        let c0 = p_cb.log_value_aligned(&[b, 0]);
        let c1 = p_cb.log_value_aligned(&[b, 1]);
        let c = sample_from_log_probs(&[c0, c1]);

        rows.push(vec![Some(a), Some(b), Some(c)]);
    }
    Ok(rows)
}

pub fn drop_some(mut data: Vec<Vec<Option<usize>>>, p_missing: f32) -> Vec<Vec<Option<usize>>> {
    for row in &mut data {
        for v in row.iter_mut() {
            if random_provider::range(0.0..1.0) < p_missing {
                *v = None;
            }
        }
    }
    data
}

fn main() {
    random_provider::set_seed(40);

    // let dists = vec![
    //     vec![0.7, 0.3],           // Var0 card 2
    //     vec![0.2, 0.3, 0.5],      // Var1 card 3
    //     vec![0.6, 0.4],           // Var2 card 2
    //     vec![0.1, 0.2, 0.3, 0.4], // Var3 card 4
    // ];

    // let data = make_iid_dataset(500, &dists);
    // println!("Data sample:");
    let data = sample_chain_dataset(1000).expect("sampling failed");
    let data = drop_some(data, 0.15);

    // your fitness objective (exact-ish for small graphs)

    for row in data.iter().take(5) {
        println!("{:?}", row);
    }
    let data = PgmDataSet::new(data);

    // let data = PgmDataSet::new(vec![
    //     vec![Some(0), Some(1), Some(0)],
    //     vec![Some(1), Some(1), Some(0)],
    //     vec![Some(0), Some(0), Some(1)],
    //     vec![Some(1), Some(0), Some(1)],
    //     vec![Some(0), Some(1), Some(1)],
    //     vec![Some(1), Some(1), Some(1)],
    // ]);

    // 1) Define the PGM structure
    //   - 3 variables, each with 2 states - binary variables
    let cards = vec![2, 2, 2];

    // 2) Build the codec for PGMs with 6 factors and max scope size 3
    let codec = PgmCodec::new(&cards, 4, 2);

    println!("Starting evolution...");
    println!("CHROMOSOME EXAMPLE:");
    let example = codec.encode();
    for f in &example[0].factors {
        println!("{:?}", f);
    }

    // 3) Define the fitness function - log-likelihood on the data
    let fitness = PgmLogLik::new(data.clone());

    let engine = GeneticEngine::builder()
        .codec(codec)
        // .fitness_fn(PgmNll {
        //     data: data.rows.clone(),
        // })
        .raw_fitness_fn(fitness)
        .minimizing()
        .alter(alters!(
            PgmScopeMutator::new(0.05, 3),
            PgmParamMutator::new(
                0.50, // half the factors get touched per individual
                0.10, // mutate ~10% of table entries
                0.25, // jitter magnitude
            )
        ))
        .build();

    let result = engine
        .iter()
        .logging()
        .take(300)
        .last()
        .inspect(|generation| {
            println!("{}", generation.metrics().dashboard());
            for i in generation.value().iter() {
                println!("Factor  {:?}", i);
                println!("");
            }
        })
        .expect("evolution failed");

    let chrom = &result.population()[0].genotype()[0];
    let m_c = marginal_ve(&chrom, &[VarId(2)], None).expect("marginalization failed");

    // turn the two log values into a proper probability vector
    let mut lp = vec![m_c.log_value_aligned(&[0]), m_c.log_value_aligned(&[1])];

    // log-softmax in place (so exp(lp).sum() == 1)
    radiate::log_normalize_in_place(&mut lp);

    println!("P(C=0)={}, P(C=1)={}", lp[0].exp(), lp[1].exp());

    // let chrom = &result.population()[0].genotype()[0];
}

// let a = VarSpec::new(0, 2);
// let b = VarSpec::new(1, 2);

// let card = |v: VarId| match v.0 {
//     0 => 2,
//     1 => 2,
//     _ => 0,
// };

// // Build an Ising factor on (A,B)
// let ising = IsingKernel { a, b };
// let f_ab = ising.build(&[0.0, 0.1, 0.0, 0.2, 0.5, -0.5]).unwrap();

// // Build a CPT P(A) as a CPT with no parents (scope [A])
// let prior = CptKernel {
//     parents: vec![],
//     child: a,
// };
// let p_a = prior.build(&[0.0, 1.0]).unwrap();

// // Compute marginal P(B) by eliminating A from P(A)*f(A,B)
// let out = variable_elimination(vec![p_a, f_ab], &[VarId(0)], &card).unwrap();
// println!("scope={:?}, logp={:?}", out.scope(), out.logp());

// // 1) logZ sanity
// let z = logz(chrom).expect("logz failed");
// println!("logZ = {z}");

// // 2) how CPT-like are the factors?
// // compute avg row-sum error if each factor treated as CPT with child = last var in its scope
// let mut worst = 0.0f32;
// let mut avg = 0.0f32;
// let mut nrows = 0usize;

// for g in chrom.factors.iter() {
//     // Only meaningful if scope non-empty
//     if g.scope.is_empty() {
//         continue;
//     }

//     let child = *g.scope.last().unwrap();

//     // Convert to DiscreteFactor and normalize rows, then check row sums of exp() are ~1
//     let mut f = gene_to_discrete(chrom, g).expect("gene_to_discrete");
//     // measure before normalization: row sums could be anything
//     // normalize then check (if it’s CPT-like, normalization shouldn’t destroy the semantics too much)
//     f.normalize_rows(child).ok();

//     // check row sums == 1
//     let axis = f.axis_of(child).unwrap();
//     let child_card = f.dims()[axis];

//     // enumerate parent assignments (all axes except child)
//     let mut parent_axes = Vec::new();
//     let mut parent_dims = Vec::new();
//     for (ax, &v) in f.scope().iter().enumerate() {
//         if v != child {
//             parent_axes.push(ax);
//             parent_dims.push(f.dims()[ax]);
//         }
//     }

//     let rows = prod_usize(&parent_dims);
//     for ridx in 0..rows {
//         // decode parents
//         let mut asg = vec![0usize; f.scope().len()];
//         let mut t = ridx;
//         for (k, &ax) in parent_axes.iter().enumerate() {
//             let d = parent_dims[k];
//             asg[ax] = t % d;
//             t /= d;
//         }

//         // sum over child
//         let mut s = 0.0f32;
//         for c in 0..child_card {
//             asg[axis] = c;
//             s += f.log_value_aligned(&asg).exp();
//         }
//         let err = (s - 1.0).abs();
//         avg += err;
//         worst = worst.max(err);
//         nrows += 1;
//     }
// }

// if nrows > 0 {
//     println!(
//         "CPT row-sum error: avg={} worst={}",
//         avg / nrows as f32,
//         worst
//     );
// }
