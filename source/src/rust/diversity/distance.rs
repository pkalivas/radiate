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

    // c4 — coefficient for matching genes whose ops differ (default 1.0)
    let diversity = NeatDistance::new(1.0, 1.0, 0.4).with_op_mismatch(0.5);
    // --8<-- [end:neat_distance]

    // --8<-- [start:tree_distance]
    // Each level down counts 1/k as much as the one above (default k = 2)
    let diversity = TreeDistance::new().with_k(2.0);
    // --8<-- [end:tree_distance]
}
