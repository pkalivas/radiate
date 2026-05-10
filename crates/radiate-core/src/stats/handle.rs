/// Declare a struct of `MetricIdx` handles that are lazily resolved against a
/// [`MetricSet`](crate::MetricSet) on first use.
///
/// Two field forms are supported:
/// - `name = expr` — a single scalar metric, resolves to a `MetricIdx`.
/// - `name[] = expr` — an indexed metric family. Resolves to `Vec<MetricIdx>`
///   sized from the `dim` argument passed to `ensure`. Each child name is
///   `format!("{}.{}", expr, i)`.
///
/// Generated API:
/// - `pub const fn new() -> Self` — all handles start `MetricIdx::INVALID`, all
///   vecs empty, `initialized = false`.
/// - `pub fn ensure(&mut self, set: &mut MetricSet, dim: usize)` — resolves
///   every handle on first call; no-op on subsequent calls.
/// - `pub fn is_initialized(&self) -> bool`.
/// - `impl Default` returning `Self::new()`.
///
/// # Example
///
/// ```ignore
/// use radiate_core::{define_metric_handles, metric_names};
///
/// define_metric_handles! {
///     pub struct AuditHandles {
///         carryover_rate = metric_names::CARRYOVER_RATE,
///         survivor_count = metric_names::SURVIVOR_COUNT,
///         scores[]       = metric_names::SCORES,
///         unique_scores[] = metric_names::UNIQUE_SCORES,
///     }
/// }
///
/// // In a step:
/// // self.handles.ensure(metrics, self.objective.dims());
/// // metrics.upsert_at(self.handles.carryover_rate, rate);
/// // metrics.upsert_at(self.handles.scores[i], &dim_vec);
/// ```
#[macro_export]
macro_rules! define_metric_handles {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::define_metric_handles!(@parse
            [$(#[$meta])* $vis $name]
            []
            []
            $($body)*
        );
    };

    // Indexed field followed by more entries.
    (@parse
        [$($head:tt)*]
        [$($scalars:tt)*]
        [$($indexed:tt)*]
        $field:ident [] = $expr:expr, $($rest:tt)*
    ) => {
        $crate::define_metric_handles!(@parse
            [$($head)*]
            [$($scalars)*]
            [$($indexed)* ($field, $expr),]
            $($rest)*
        );
    };

    // Indexed field as final entry (no trailing comma).
    (@parse
        [$($head:tt)*]
        [$($scalars:tt)*]
        [$($indexed:tt)*]
        $field:ident [] = $expr:expr
    ) => {
        $crate::define_metric_handles!(@parse
            [$($head)*]
            [$($scalars)*]
            [$($indexed)* ($field, $expr),]
        );
    };

    // Scalar field followed by more entries.
    (@parse
        [$($head:tt)*]
        [$($scalars:tt)*]
        [$($indexed:tt)*]
        $field:ident = $expr:expr, $($rest:tt)*
    ) => {
        $crate::define_metric_handles!(@parse
            [$($head)*]
            [$($scalars)* ($field, $expr),]
            [$($indexed)*]
            $($rest)*
        );
    };

    // Scalar field as final entry.
    (@parse
        [$($head:tt)*]
        [$($scalars:tt)*]
        [$($indexed:tt)*]
        $field:ident = $expr:expr
    ) => {
        $crate::define_metric_handles!(@parse
            [$($head)*]
            [$($scalars)* ($field, $expr),]
            [$($indexed)*]
        );
    };

    // Terminal arm: emit the struct + impl.
    (@parse
        [$(#[$meta:meta])* $vis:vis $name:ident]
        [$(($s_field:ident, $s_expr:expr),)*]
        [$(($i_field:ident, $i_expr:expr),)*]
    ) => {
        $(#[$meta])*
        $vis struct $name {
            $(pub $s_field: $crate::stats::MetricIdx,)*
            $(pub $i_field: ::std::vec::Vec<$crate::stats::MetricIdx>,)*
            initialized: bool,
        }

        impl $name {
            #[inline]
            pub const fn new() -> Self {
                Self {
                    $($s_field: $crate::stats::MetricIdx::INVALID,)*
                    $($i_field: ::std::vec::Vec::new(),)*
                    initialized: false,
                }
            }

            #[inline]
            pub fn is_initialized(&self) -> bool {
                self.initialized
            }

            pub fn ensure(
                &mut self,
                set: &mut $crate::MetricSet,
                #[allow(unused_variables)] dim: usize,
            ) {
                if self.initialized {
                    return;
                }
                $(self.$s_field = set.resolve(&$s_expr);)*
                $(
                    self.$i_field = (0..dim)
                        .map(|__i| {
                            let __name: $crate::SmallStr =
                                ::std::format!("{}.{}", $i_expr, __i).into();
                            set.resolve(&__name)
                        })
                        .collect();
                )*
                self.initialized = true;
            }
        }

        impl ::std::default::Default for $name {
            #[inline]
            fn default() -> Self {
                Self::new()
            }
        }
    };
}
