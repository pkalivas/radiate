
!!! info "Core Library Gene Implementations"

    === "BitGene"

        ```rust
        #[derive(Clone, PartialEq)]
        pub struct BitGene {
            allele: bool,
        }
        ```

        * **Allele**: `bool`
        * **Description**: Represents a single bit `true`/`false`
        * **Implements**: `Gene`

    === "FloatGene"

        ```rust
        #[derive(Clone, PartialEq)]
        pub struct FloatGene {
            pub allele: f32,
            pub min: f32,
            pub max: f32,
            pub upper_bound: f32,
            pub lower_bound: f32,
        }
        ```

        * **Allele**: `f32`
        * **Description**: Represents a single floating-point number
        * **Implements**: `Gene`, `NumericGene`, `BoundedGene`

    === "IntGene"

        ```rust
        #[derive(Clone, PartialEq)]
        pub struct IntGene<T: Integer<T>>
        where
            Standard: rand::distributions::Distribution<T>,
        {
            pub allele: T,
            pub min: T,
            pub max: T,
            pub upper_bound: T,
            pub lower_bound: T,
        }
        ```

        * **Allele**: `I` where `I` implements `Integer<I>`. `Integer` is a trait in Radiate and is implemented for `i8`, `i16`, `i32`, `i64`, `i128`.
        * **Description**: Represents a single integer number
        * **Implements**: `Gene`, `NumericGene`, `BoundedGene`

    === "CharGene"

        ```rust
        #[derive(Clone, PartialEq)]
        pub struct CharGene {
            pub allele: char,
        }
        ```

        * **Allele**: `char`
        * **Description**: Represents a single character
        * **Implements**: `Gene`

    === "PermutationGene"

        ```rust
        #[derive(Debug, Clone, PartialEq)]
        pub struct PermutationGene<A: PartialEq + Clone> {
            pub index: usize,
            pub alleles: Arc<Vec<A>>,
        }
        ```

        * **Allele**: `A`
        * **Description**: Given a list of `A`, represents a single value of the list
        * **Implements**: `Gene`