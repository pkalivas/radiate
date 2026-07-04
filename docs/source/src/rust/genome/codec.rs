use radiate::prelude::*;
use std::sync::Arc;

fn main() {}

fn float_codec() {
    // --8<-- [start:float_codec]
    // single float parameter
    let codec_scalar: FloatCodec<f32> = FloatCodec::scalar(-1.0..1.0).with_bounds(-10.0..10.0);
    let encoded_scalar: Genotype<FloatChromosome<f32>> = codec_scalar.encode();
    let decoded_scalar: f32 = codec_scalar.decode(&encoded_scalar);

    // vector of 5 floats
    let codec_vector: FloatCodec<f64, Vec<f64>> =
        FloatCodec::vector(5, -1.0..1.0).with_bounds(-10.0..10.0);
    let encoded_vector: Genotype<FloatChromosome<f64>> = codec_vector.encode();
    let decoded_vector: Vec<f64> = codec_vector.decode(&encoded_vector);

    // 3x2 matrix of floats - 3 Chromosomes each with 2 genes
    let codec_matrix: FloatCodec<f32, Vec<Vec<f32>>> =
        FloatCodec::matrix(vec![2, 2, 2], -0.1..0.1).with_bounds(-1.0..1.0);
    let encoded_matrix: Genotype<FloatChromosome<f32>> = codec_matrix.encode();
    let decoded_matrix: Vec<Vec<f32>> = codec_matrix.decode(&encoded_matrix);

    /*
    Now if you need a more complex structure, most chromosomes themselves can be input as codecs to
    the engine, allowing you to create weirdly structured codecs like below

    Below we create a codec that will encode:
    [
        [A chromosome of 2 float genes with init ranges=(-1.0..1.0) bounds=(-10.0..10.0)]
        [A chromosome of 5 float genes with init ranges=(-0.5..0.5) bounds=(-5.0..5.0)]
        [A chromosome of 2 float genes with, one with init ranges=(-1.0..1.0) bounds=(-10.0..10.0)
            and the other with init ranges=(-0.5..0.5) bounds=(-5.0..5.0)]
    ]
     */
    let jagged_codec = vec![
        FloatChromosome::from((2, -1.0..1.0, -10.0..10.0)),
        FloatChromosome::from((5, -0.5..0.5, -5.0..5.0)),
        FloatChromosome::from(vec![
            FloatGene::new(1.0, -1.0..1.0, -10.0..10.0), // <- initial allele of 1.0, but will be randomly generated between -1.0 and 1.0 during encoding
            FloatGene::from((-0.5..0.5, -5.0..5.0)), // <- randomly generated allele between -0.5 and 0.5 with bounds between -5.0 and 5.0
        ]),
    ];
    // --8<-- [end:float_codec]
}

fn int_codec() {
    // --8<-- [start:int_codec]
    // single int parameter
    let codec_scalar: IntCodec<i32> = IntCodec::scalar(-10..10).with_bounds(-100..100);
    let encoded_scalar: Genotype<IntChromosome<i32>> = codec_scalar.encode();
    let decoded_scalar: i32 = codec_scalar.decode(&encoded_scalar);

    // vector of 5 ints
    let codec_vector: IntCodec<u8, Vec<u8>> = IntCodec::vector(5, 0..255).with_bounds(0..255);
    let encoded_vector: Genotype<IntChromosome<u8>> = codec_vector.encode();
    let decoded_vector: Vec<u8> = codec_vector.decode(&encoded_vector);

    // 3x2 matrix of ints - 3 Chromosomes each with 2 genes
    let codec_matrix: IntCodec<i16, Vec<Vec<i16>>> =
        IntCodec::matrix(vec![2, 2, 2], -10..10).with_bounds(-100..100);
    let encoded_matrix: Genotype<IntChromosome<i16>> = codec_matrix.encode();
    let decoded_matrix: Vec<Vec<i16>> = codec_matrix.decode(&encoded_matrix);

    /*
    Now if you need a more complex structure, most chromosomes themselves can be input as codecs to
    the engine, allowing you to create weirdly structured codecs like below

    Below we create a codec that will encode:
    [
        [A chromosome of 2 int genes with init ranges=(-1..1) bounds=(-10..10)]
        [A chromosome of 5 int genes with init ranges=(-1..1) bounds=(-5..5)]
        [A chromosome of 2 int genes with, one with init ranges=(-1..1) bounds=(-10..10)
            and the other with init ranges=(-1..1) bounds=(-5..5)]
    ]
     */
    let jagged_codec = vec![
        IntChromosome::from((2, -1..1, -10..10)),
        IntChromosome::from((5, -1..1, -5..5)),
        IntChromosome::from(vec![
            IntGene::new(0, -1..1, -10..10),
            IntGene::new(0, -1..1, -5..5),
        ]),
    ];
    // --8<-- [end:int_codec]
}

fn char_codec() {
    // --8<-- [start:char_codec]
    // single char parameter

    // vector of 5 chars - specify the char set
    let codec_vector = CharCodec::vector(5)
        .with_char_set("abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<char>>());
    let encoded_vector: Genotype<CharChromosome> = codec_vector.encode();
    let decoded_vector: Vec<char> = codec_vector.decode(&encoded_vector);

    // 3x2 matrix of chars
    let codec_matrix = CharCodec::matrix(3, 2);
    let encoded_matrix: Genotype<CharChromosome> = codec_matrix.encode();
    let decoded_matrix: Vec<Vec<char>> = codec_matrix.decode(&encoded_matrix);
    // --8<-- [end:char_codec]
}

fn bit_codec() {
    // --8<-- [start:bit_codec]
    // single bit parameter - not sure why anyone would ever want this, but here it is.
    let codec_scalar = BitCodec::scalar();
    let encoded_scalar: Genotype<BitChromosome> = codec_scalar.encode();
    let decoded_scalar: bool = codec_scalar.decode(&encoded_scalar);

    // vector of 5 bits
    let codec_vector = BitCodec::vector(5);
    let encoded_vector: Genotype<BitChromosome> = codec_vector.encode();
    let decoded_vector: Vec<bool> = codec_vector.decode(&encoded_vector);

    // 3x2 matrix of bits - 3 Chromosomes each with 2 genes
    let codec_matrix = BitCodec::matrix(3, 2);
    let encoded_matrix: Genotype<BitChromosome> = codec_matrix.encode();
    let decoded_matrix: Vec<Vec<bool>> = codec_matrix.decode(&encoded_matrix);
    // --8<-- [end:bit_codec]
}

fn subset_codec() {
    // --8<-- [start:subset_codec]
    #[derive(Debug, Clone)]
    pub struct Item {
        pub weight: f32,
        pub value: f32,
    }

    let items = vec![
        Item {
            weight: 2.0,
            value: 3.0,
        },
        Item {
            weight: 3.0,
            value: 4.0,
        },
        Item {
            weight: 4.0,
            value: 5.0,
        },
        //...
    ];

    let subset_codec = SubSetCodec::new(items);

    let genotype: Genotype<BitChromosome> = subset_codec.encode();
    let decoded: Vec<Arc<Item>> = subset_codec.decode(&genotype);
    // --8<-- [end:subset_codec]
}

fn permutation_codec() {
    // --8<-- [start:permutation_codec]
    #[derive(Debug, Clone, PartialEq)]
    pub struct WayPoint {
        pub x: f32,
        pub y: f32,
    }

    let waypoints = vec![
        WayPoint { x: 1.0, y: 2.0 },
        WayPoint { x: 3.0, y: 4.0 },
        WayPoint { x: 5.0, y: 6.0 },
        WayPoint { x: 7.0, y: 8.0 },
    ];

    // Encode a genotype of Genotype<PermutationChromosome> and decode to a Vec<WayPoint>
    // where each WayPoint is a unique item from the original set of waypoints.
    // This will ensure that the permutation is valid and does not contain duplicates.
    let codec = PermutationCodec::new(waypoints.to_vec());
    let encoded: Genotype<PermutationChromosome<WayPoint>> = codec.encode();
    let decoded: Vec<WayPoint> = codec.decode(&encoded);
    // --8<-- [end:permutation_codec]
}

fn fn_codec() {
    // --8<-- [start:fn_codec]
    const N_QUEENS: usize = 8;

    struct NQueens(Vec<i8>);

    // this is a simple example of the NQueens problem.
    // The resulting codec type will be FnCodec<IntChromosome<i8>, NQueens>.
    let codec: FnCodec<IntChromosome<i8>, NQueens> = FnCodec::new()
        .with_encoder(|| {
            Genotype::from(IntChromosome::new(
                (0..N_QUEENS)
                    .map(|_| IntGene::from(0..N_QUEENS as i8))
                    .collect(),
            ))
        })
        .with_decoder(|genotype| {
            NQueens(
                genotype[0]
                    .as_slice()
                    .iter()
                    .map(|g| *g.allele())
                    .collect::<Vec<i8>>(),
            )
        });

    // encode and decode
    let genotype: Genotype<IntChromosome<i8>> = codec.encode();
    let decoded: NQueens = codec.decode(&genotype);
    // --8<-- [end:fn_codec]
}
