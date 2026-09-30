use radiate::prelude::*;

fn main() {
    // --8<-- [start:hamming]
    let diversity = HammingDistance;
    // --8<-- [end:hamming]

    // --8<-- [start:euclidean]
    let diversity = EuclideanDistance;
    // --8<-- [end:euclidean]

    // --8<-- [start:cosine]
    let diversity = CosineDistance;
    // --8<-- [end:cosine]

    // --8<-- [start:neat_distance]
    // c1, c2, c3 — coefficients for excess genes, disjoint genes,
    // and average weight differences respectively.
    let diversity = NeatDistance::new(1.0, 1.0, 0.4);
    // --8<-- [end:neat_distance]
}
