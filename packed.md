# `PackedBitChromosome`: bit strings at scale

Plan, rewritten 2026-09-27. Replaces `bitflip.md`, which covered bit flipping on `IntChromosome`, and `TODO.md`, which held the discussion before that.

## Goal

Radiate should handle large bit-string problems well: 10⁵–10⁶+ bits per chromosome, large populations, long runs. The operators should behave correctly at the bit level, and the API shouldn't surprise anyone.

`BitChromosome` stays as the simple default. `PackedBitChromosome` is an opt-in type for scale: same problems, same results, 8× less memory, and operators that work 64 bits at a time.

## Decisions so far

| Decision | Why |
|---|---|
| **Drop bit flipping on `IntChromosome`.** | Bit-flipping *bounded* integers is something no other library does (jMetal, DEAP, pymoo, Jenetics and ECJ all mutate bounded ints by value). It needed an offset-space encoding plus a rejection policy just to stay in bounds, and it runs into Hamming cliffs. The value-level mutators already cover bounded ints. The one real use case, "a `u64` used as 64 bits", is a bit string, and this type handles it properly. |
| **`BitFlipMutator` is implemented per concrete chromosome.** | Already done: `impl Mutate<BitChromosome>` replaced the generic `C: ContiguousChromosome<Gene = BitGene>` impl. Rust's overlap check doesn't treat different associated `Gene` types as disjoint, so this is what allows a second `impl Mutate<PackedBitChromosome>`. |
| **Keep `BitChromosome`.** | Existing code keeps working, and it's the simpler type for short strings. Packing is opt-in. Whether packing ever becomes the default above some size is decided later, based on measurements. |
| **Tail bits are unspecified, and every read masks them.** | Required so any bit length works, not just multiples of 64. |
| **Bit-level operators ship in v1** (not "later"). | Without them, the only crossovers and distances available on this type work on whole words. See "The word-level trap". |
| **Mutation and crossover counts are in bits.** | Same numbers as `BitChromosome`, so switching types doesn't change the metrics. |

## What packing buys

Memory for **100,000 bits**, one chromosome:

| Representation | Bytes | Bits of memory per bit |
|---|---|---|
| `BitChromosome` (`Vec<BitGene>`, 1 byte each) | 100,000 | 8 |
| `PackedBitChromosome` (1,563 × `u64`) | 12,504 | 1 |

That cost is paid for every individual, again in the per-generation ecosystem clone (`Generation::from` deep-clones after `pipeline.run()`), and again in checkpoint size.

Packing also allows **word-level operations**, 64 bits per instruction:

- bit flip by XOR
- uniform crossover by mask: `d = (a ^ b) & m; a ^= d; b ^= d`
- Hamming distance by `(a ^ b).count_ones()`
- OneMax-style fitness by `count_ones`

Jenetics packs its storage but unpacks it into a reference array for every alteration, so it gets the memory savings and none of the speed. We can get both.

## Design

### The key idea: the gene is a word

`BitChromosome` can't be repacked in place. `iter_mut`, `get_mut` and `as_mut_slice` hand out `&mut BitGene`, and you can't point a reference at a single bit. Doing that would mean proxy types or splitting `Chromosome` into read and write traits, which touches every operator.

So we **change what a gene is, not how genes are stored**. If the gene is a whole `u64` word, then `&mut Gene` is a real reference to real memory, and `Chromosome` / `ContiguousChromosome` can be implemented as they are, with no proxies and no trait changes. The bit-level API lives on the chromosome as its own methods, the same way `BitChromosome` has `pack`/`unpack`.

Consequences:

1. **`Chromosome::len()` counts words**, not bits. The bit count is `num_bits()`. Anything generic that reports gene counts (metrics, generic operators) sees words.
2. **Bit order:** bit `i` lives in word `i >> 6`, at bit `i & 63`. That's least-significant first, the same default as `fixedbitset` and `bitvec`. `BitChromosome::pack` is MSB-first. The conversion between the two types handles that, and nothing else shares words between them.
3. **The tail.** When `num_bits % 64 != 0`, the last word has unused high bits. Generic operators can't keep them clean: `UniformMutator` randomizes whole words, and `Swap` can move a full word into the last position. The invariant is therefore: **bits at positions `>= num_bits` are unspecified.** Everything that reads bits masks the tail (`count_ones`, `PartialEq`, bit iterators, distance), and the codec clears the tail before handing a value to the fitness function.
   - If a generic operator moves the tail word into the middle, its unused high bits become ordinary bits. They're random, but they're valid bits, so nothing breaks.
   - Rejected: storing the valid-bit width in each gene. It doubles the gene size, and it breaks as soon as a partial word gets moved.
   - Rejected: requiring `num_bits % 64 == 0`. It's simpler, but it doesn't cover arbitrary problem sizes.

### Gene: `radiate-core/src/genome/chromosomes/packed.rs`

```rust
/// 64 bits of a [`PackedBitChromosome`]. The gene is the word, not the bit, so
/// `&mut BitWord` is a real reference and the chromosome can implement
/// [`ContiguousChromosome`] without proxies.
#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[repr(transparent)]
pub struct BitWord(u64);

impl Valid for BitWord {}

impl Gene for BitWord {
    type Allele = u64;

    fn allele(&self) -> &u64 { &self.0 }
    fn allele_mut(&mut self) -> &mut u64 { &mut self.0 }

    /// A uniformly random word: 64 independent fair coins.
    fn new_instance(&self) -> Self { BitWord(random_provider::random()) }

    fn with_allele(&self, allele: &u64) -> Self { BitWord(*allele) }
    fn set_allele(&mut self, allele: u64) { self.0 = allele; }
}
```

### Chromosome

```rust
const WORD_BITS: usize = 64;

/// A bit string stored 64 bits per [`BitWord`] gene.
///
/// Generic operators see 64-bit words. Bit-level access goes through the methods
/// below. Bits at positions `>= num_bits` in the last word are unspecified, and
/// every bit-level reader masks them.
#[derive(Clone, Default, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PackedBitChromosome {
    words: Vec<BitWord>,
    num_bits: usize,
}

impl PackedBitChromosome {
    /// A random bit string of `num_bits` fair coins.
    pub fn new(num_bits: usize) -> Self;

    pub fn num_bits(&self) -> usize;

    pub fn bit(&self, i: usize) -> bool;
    pub fn set_bit(&mut self, i: usize, value: bool);
    pub fn flip_bit(&mut self, i: usize);

    /// Tail-masked popcount.
    pub fn count_ones(&self) -> usize;
    pub fn iter_bits(&self) -> impl Iterator<Item = bool> + '_;

    /// Zero the unspecified bits past `num_bits`.
    pub fn clear_tail(&mut self);

    /// Mask of the valid bits in the last word (`u64::MAX` when `num_bits % 64 == 0`).
    fn tail_mask(&self) -> u64;
}

impl FromIterator<bool> for PackedBitChromosome { /* push bits, track num_bits */ }
impl From<&BitChromosome> for PackedBitChromosome { /* ... */ }
impl From<&PackedBitChromosome> for BitChromosome { /* ... */ }

/// Equality ignores the unspecified tail bits.
impl PartialEq for PackedBitChromosome { /* compare full words, then last word under tail_mask */ }

impl Valid for PackedBitChromosome {}

impl Chromosome for PackedBitChromosome {
    type Gene = BitWord;
    // iter / iter_mut / get / get_mut / set over `words`
    /// Number of words (genes), not bits. See [`Self::num_bits`].
    fn len(&self) -> usize { self.words.len() }
}

impl ContiguousChromosome for PackedBitChromosome {
    fn as_slice(&self) -> &[BitWord] { &self.words }
    fn as_mut_slice(&mut self) -> &mut [BitWord] { &mut self.words }
}
```

### Codec: `radiate-core/src/codecs/packed.rs`

```rust
/// Encodes random bit strings of `num_bits` each. Decodes to the chromosome with its
/// tail cleared, so the fitness function gets the bit-level API (`bit`, `count_ones`,
/// `iter_bits`) and never sees the tail.
pub struct PackedBitCodec { num_bits: usize }

impl Codec<PackedBitChromosome, PackedBitChromosome> for PackedBitCodec {
    fn encode(&self) -> Genotype<PackedBitChromosome> {
        Genotype::new(vec![PackedBitChromosome::new(self.num_bits)])
    }

    fn decode(&self, genotype: &Genotype<PackedBitChromosome>) -> PackedBitChromosome {
        let mut bits = genotype[0].clone(); // memcpy of num_bits / 8 bytes
        bits.clear_tail();
        bits
    }
}

// OneMax:  |bits: PackedBitChromosome| bits.count_ones() as f32
```

A `Vec<bool>` decode is left out on purpose: it unpacks on every evaluation and throws away the benefit. A multi-chromosome (matrix) variant follows `BitCodec`'s pattern if someone needs it.

## Operators

### The word-level trap

These operators are **blanket impls**, so they compile against `PackedBitChromosome` and silently act on whole 64-bit words:

| Operator | Blanket bound | On packed |
|---|---|---|
| `UniformCrossover` | `C: Chromosome` | swaps whole words |
| `MultiPointCrossover` | `C: ContiguousChromosome` | cuts only at word boundaries |
| `ShuffleCrossover` | `C: ContiguousChromosome + Clone` | shuffles words |
| `UniformMutator` | `C: Chromosome` | re-randomizes 64 bits at a time |
| `SwapMutator`, `ScrambleMutator`, `InversionMutator` | `C: ContiguousChromosome` | move 64-bit blocks |
| `HammingDistance` | `G::Allele: PartialEq` | counts differing *words* |

None of these can be specialized or blocked at the type level, so the fix has two parts:

1. **Ship a complete set of bit-level operators in v1**, so there's always a correct choice.
2. **Documentation that leads with those operators.** The rustdoc on `PackedBitChromosome` and the docs page list the bit-level operators first and say plainly which generic operators act on words.

Python doesn't have this problem, because operators go through a registry for each chromosome type (see Python).

### Bit-level operator set (v1)

Because the generic operators above are blanket impls, the bit-level versions have to be **new types**, not extra impls on the existing ones. `BitFlipMutator` is the exception: it's already implemented per concrete chromosome.

| Operator | Kind | Word-level technique |
|---|---|---|
| `BitFlipMutator` | new impl on the existing type | `bernoulli_indices` over `num_bits` and `^= 1 << (k & 63)`. Random draws scale with the number of flips. |
| `BitUniformCrossover` (name TBD) | new type | Collect the selected positions into one mask per word, then `d = (a ^ b) & mask`. At `p == 0.5`: one `random::<u64>()` per word (64 fair coins in one draw), with the tail masked on the last word. |
| `BitMultiPointCrossover` (name TBD) | new type | Sample cut points from `1..num_bits` with `sample_indices`. Swap whole words inside each segment, and apply a partial mask on the word each cut falls in. |
| `BitHammingDistance` (name TBD) | new `Diversity` type | `Σ (a ^ b).count_ones()` over words, last word masked, normalized by `num_bits`. |

#### `BitFlipMutator`

```rust
impl Mutate<PackedBitChromosome> for BitFlipMutator {
    fn rates(&self) -> RateSet { RateSet::new(self.rate.clone()) }

    #[inline]
    fn mutate_chromosome(&mut self, chromosome: &mut PackedBitChromosome, ctx: &mut AlterContext) -> usize {
        let p = ctx.rate();
        let num_bits = chromosome.num_bits();
        let words = chromosome.as_mut_slice();

        let mut flips = 0;
        random_provider::with_rng(|rng| {
            rng.bernoulli_indices(p, num_bits, |k| {
                *words[k >> 6].allele_mut() ^= 1 << (k & 63);
                flips += 1;
            });
        });

        flips
    }
}
```

The closure doesn't use `random_provider`, so the `RdRand` form (with the RNG borrowed for the whole loop) is safe here.

#### `BitUniformCrossover`

```rust
fn cross_chromosomes(&self, one: &mut PackedBitChromosome, two: &mut PackedBitChromosome, ctx: &mut AlterContext) -> usize {
    let num_bits = one.num_bits().min(two.num_bits());
    let (a, b) = (one.as_mut_slice(), two.as_mut_slice());

    let mut swap = |w: usize, mask: u64| {
        let d = (a[w].allele() ^ b[w].allele()) & mask;
        *a[w].allele_mut() ^= d;
        *b[w].allele_mut() ^= d;
    };

    // Positions arrive in ascending order: build one mask per word and apply it
    // when the next position moves to a new word.
    let (mut cur, mut mask, mut swapped) = (0, 0u64, 0);
    random_provider::with_rng(|rng| {
        rng.bernoulli_indices(ctx.rate(), num_bits, |k| {
            if k >> 6 != cur {
                swap(cur, mask);
                (cur, mask) = (k >> 6, 0);
            }
            mask |= 1 << (k & 63);
            swapped += 1;
        });
    });
    swap(cur, mask);

    swapped
}
```

#### Could these also accept `BitChromosome`?

Yes. Each new operator is its own type, so each can also have a concrete `impl ... for BitChromosome`, and the "Bit*" operators then work on both representations. That's nice for users (switching storage never means switching operators), but it's extra surface area. **Default: packed only in v1.** Add the `BitChromosome` impls if the docs end up reading awkwardly without them.

### Operator matrix for `PackedBitChromosome`

| Operator | Granularity | Recommended |
|---|---|---|
| `BitFlipMutator` | bit | yes |
| `BitUniformCrossover` | bit | yes |
| `BitMultiPointCrossover` | bit | yes |
| `BitHammingDistance` | bit | yes (speciation) |
| `UniformCrossover`, `MultiPointCrossover`, `ShuffleCrossover` | word | no, use the bit versions |
| `UniformMutator` | word (re-randomizes 64 bits) | no |
| `SwapMutator`, `ScrambleMutator`, `InversionMutator` | word (moves blocks) | no |
| `HammingDistance` | word | no, use `BitHammingDistance` |

## Rates

- The rate for `BitFlipMutator` is **per bit**, for both chromosome types.
- The standard default in the literature is `p ≈ 1 / num_bits`, so one expected flip per chromosome. It depends on the chromosome length, so no fixed default fits every size. Python's `Mutate.bit_flip(rate=0.1)` default is reasonable at 10 bits and extreme at 10⁶ bits (100,000 flips per generation). The docs should give the `1/L` rule up front.

## Python

### What the user sees

Packing changes storage, not meaning, so it's a flag, not a new class:

```python
engine = rd.Engine.bit(100_000, packed=True, use_numpy=True)   # fitness gets an np.bool_ array
engine = rd.Engine.bit(100_000, use_numpy=True)                # today: BitChromosome, same fitness

codec = rd.BitCodec(100_000, packed=True, use_numpy=True)
```

- **It's the existing pattern.** `Engine.int`, `Engine.graph` and `Engine.tree` already choose among Rust chromosome types with a parameter (`dtype`).
- **Default `packed=False`.** Opt-in, and there's no automatic switch based on length.
- **Decode matches `BitCodec`**, so an existing fitness function works unchanged. `list[bool]` builds 10⁵ Python objects per evaluation, so the docs point large problems to `use_numpy=True` (built in Rust, about 100 KB at 100k bits).

### Genes show bits, and the words sit behind an explicit accessor

This is the industry standard. Python's `bitarray`, Java's `BitSet`, `boost::dynamic_bitset`, and Rust's `bitvec`/`fixedbitset` all index bits and make raw words a deliberate opt-in.

`PyChromosome` holds the Rust chromosome by value (`ChromosomeInner`), and `__getitem__` already returns a converted `PyGene` copy, so Python doesn't have Rust's by-reference constraint:

- A new `ChromosomeInner::PackedBit(PackedBitChromosome)` variant with hand-written arms. The macro-generated `match_chromosome!` arms would expose words, so they skip this variant.
- `__len__` → `num_bits()`. `__getitem__(i)` → the **existing** Bit `PyGene` built from `bit(i)`. Iteration yields bits.
- Building from Python (a list of Bit genes or bools) packs them.
- `__eq__` uses `PartialEq`, which already ignores the tail.
- `.words` → numpy `uint64` with the tail cleared, for power users. It returns `None` on other chromosome types.

```python
chrom = result.population()[0].genotype[0]

len(chrom)         # 100            - bits, not 2 words
chrom[3]           # Gene(Bit, True) - the same Bit gene BitChromosome gives you
chrom.words        # array([...], dtype=uint64) - opt-in, tail cleared
```

### Operators go through a registry

Python picks operators through a **registry for each chromosome type** (`bit_registry`, `int_registry`, …). A `packed_bit_registry()` maps:

- `Mutate.bit_flip` → `BitFlipMutator` (packed impl)
- `Cross.uniform` → `BitUniformCrossover`
- `Cross.multi_point` → `BitMultiPointCrossover`
- `Dist.hamming` → `BitHammingDistance`

It **leaves out** `Mutate.swap/scramble/inversion/uniform` and `Cross.shuffle`. Python users get bit-level behavior and never see the word-level trap.

Unsupported operators must fail with a message that names the cause (*"`Mutate.swap` is not supported for packed bit chromosomes"*), not the registry's generic "Invalid alterer type". Better: catch it in Python before the engine is built, using a `GeneType.PACKED_BIT` that `allowed_genes` checks. Gene *values* are still plain Bit genes.

## Milestones

Each milestone ends with passing tests. Milestone 2 decides whether the rest goes ahead.

### 0. Finish the leftovers from the int-bit-flip plan
- Remove `debug_assert!(p.is_finite())` from `BitFlipMutator`. `bernoulli_indices` already defines what non-finite rates do.
- Update the `BitFlipMutator` rustdoc: rate is per bit, and it's implemented for `BitChromosome` (then packed).
- CHANGELOG, Changed (breaking): `BitFlipMutator` is implemented for `BitChromosome` specifically, not for any chromosome with `Gene = BitGene`.

### 1. Core type
- `packed.rs`: `BitWord`, `PackedBitChromosome`, exports, conversions to and from `BitChromosome`.
- `PackedBitCodec`.
- `impl Mutate<PackedBitChromosome> for BitFlipMutator`.
- Tests:
  - bit API round-trips (`set_bit`/`bit`/`flip_bit`) at positions 0, 63, 64 and `num_bits - 1`
  - `count_ones` and `PartialEq` ignore garbage in the tail
  - `clear_tail` zeroes exactly the tail
  - conversion to and from `BitChromosome` round-trips, including lengths that aren't multiples of 64
  - `BitFlipMutator`: `p = 0` changes nothing, `p = 1` inverts every in-range bit, the mean flip count is `num_bits · p`, every position (including the last word) gets flipped, and the tail is never counted
  - a OneMax run through `GeneticEngine` converges

### 2. Measure (go/no-go)
There's no Rust bench harness, and we shouldn't add one for this. Time it ad hoc (a scratch example or an ignored test), running OneMax with `BitChromosome` vs `PackedBitChromosome` at **1k / 100k / 1M bits**:
- memory per individual
- the per-generation clone. Time it on the wall clock, because metric timings exclude the clone in `Generation::from`.
- generations per second, using `BitFlipMutator` and each type's best available crossover

Expected: roughly 8× less memory, and much faster clones and flips at 10⁵+. If the gains are small at realistic sizes, stop here and write down why.

### 3. Bit-level operators
- `BitUniformCrossover`, `BitMultiPointCrossover`, `BitHammingDistance`.
- Tests: counts match the rate; crossover only exchanges bits (the combined popcount of both parents is unchanged, and at every bit position the pair of parent values is preserved (only swapped)); multi-point cuts land on and inside word boundaries correctly; distance matches a naive per-bit count, including the tail.

### 4. Python
- `packed: bool = False` on `Engine.bit` and `BitCodec`, plus the `.pyi` overloads.
- `ChromosomeInner::PackedBit` with the bit-view arms, plus `.words`.
- `packed_bit_registry()`, `GeneType.PACKED_BIT`, and clear errors for unsupported operators.
- Tests: fitness gets the same shape as with `BitCodec`, indexing returns bits, `len` is bits, `.words` has the tail cleared, and unsupported operators fail before the engine is built.

### 5. Docs and release
- The docs page for bit strings covers both chromosome types, when to choose packed, the bit-level operators, the `1/L` rate rule, and the word-level trap. Prose stays language-neutral, with Rust and Python tabs.
- CHANGELOG, Added: `PackedBitChromosome`, `PackedBitCodec`, the bit-level operators, and Python `packed=True`.
- An example: large-scale OneMax (or a knapsack variant) in both languages.

## Open questions

- **Names.** `PackedBitChromosome` / `BitWord` / `PackedBitCodec` are placeholders, and so are the `Bit*` operator names. Should the operators say "Packed" instead of "Bit", given they're packed-only in v1?
- **Bit operators on `BitChromosome` too?** Default is no for v1 (see above).
- **Mutation count unit** is decided as bits. That means the count for this type isn't "genes changed", since genes are words. Document it on the type.
- **Default for `Engine.bit`.** Should it pick packed automatically above some length after milestone 2? Default: no, keep it explicit.
- **Gray coding.** Out of scope. It's a representation choice for encoding numbers in bits, not something a bit-string chromosome needs.
