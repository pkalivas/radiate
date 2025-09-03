mod builder;
mod crossover;
mod expr;
mod mutate;
mod pred;

use std::fmt;

pub use builder::*;
pub use crossover::*;
pub use expr::*;
pub use mutate::*;
pub use pred::*;
use radiate_core::Gene;

struct ExprTree<'a, G: Gene>(pub &'a Expr<G>);

impl<'a, G: Gene> fmt::Display for ExprTree<'a, G> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use std::any::type_name_of_val;

        // Shorten long Rust type paths like "my::long::module::Type" -> "Type"
        fn short_type(full: &str) -> &str {
            full.rsplit("::").next().unwrap_or(full)
        }

        fn write_node<G: Gene>(
            f: &mut fmt::Formatter<'_>,
            node: &Expr<G>,
            prefix: &str,
            is_last: bool,
        ) -> fmt::Result {
            let (branch, next_prefix) = if is_last {
                ("└─ ", format!("{prefix}   "))
            } else {
                ("├─ ", format!("{prefix}│  "))
            };

            // Build label for this node
            let label = match node {
                Expr::Seq(xs) => format!("Seq(len={})", xs.len()),
                Expr::Select(SelectExpr::All, _) => "Select::All".to_string(),
                Expr::Select(SelectExpr::Some(pred), _) => {
                    // if you wrap them with names, prefer the name
                    if let Some(name) = pred.name() {
                        format!("Select::Some({name})")
                    } else {
                        let tn = short_type(type_name_of_val(&*pred));
                        format!("Select::Some(<{tn}>)")
                    }
                }
                Expr::Index(i, _) => format!("Index({i})"),
                Expr::Filter(pred, _) => {
                    if let Some(name) = pred.name() {
                        format!("Filter({name})")
                    } else {
                        let tn = short_type(type_name_of_val(&*pred));
                        format!("Filter(<{tn}>)")
                    }
                }
                Expr::Prob(p, _) => format!("Prob({:.2})", p),
                Expr::Mut(mapper) => {
                    if let Some(name) = mapper.name() {
                        format!("Mut({name})")
                    } else {
                        let tn = short_type(type_name_of_val(&*mapper));
                        format!("Mut(<{tn}>)")
                    }
                }
                Expr::Cross(kind, _, _) => {
                    if let Some(name) = kind.name() {
                        format!("Cross({name})")
                    } else {
                        let tn = short_type(type_name_of_val(&*kind));
                        format!("Cross(<{tn}>)")
                    }
                }
                Expr::NoOp => "NoOp".to_string(),
                Expr::Fused(fused) => match fused {
                    FusedExpr::Mutate(prob, f) => {
                        let prob_str = if let Some(p) = prob {
                            format!("Some({:.2})", p)
                        } else {
                            "None".to_string()
                        };
                        if let Some(name) = f.name() {
                            format!("Fused::Mutate(prob={prob_str}, {name})")
                        } else {
                            let tn = short_type(type_name_of_val(&*f));
                            format!("Fused::Mutate(prob={prob_str}, <{tn}>)")
                        }
                    }
                    FusedExpr::None => "Fused::None".to_string(),
                },
            };

            // Print this node's label
            if prefix.is_empty() {
                // root
                writeln!(f, "{label}")?;
            } else {
                writeln!(f, "{prefix}{branch}{label}")?;
            }

            // Children for this node
            match node {
                Expr::Seq(xs) => {
                    for (i, child) in xs.iter().enumerate() {
                        write_node(f, child, &next_prefix, i + 1 == xs.len())?;
                    }
                }
                Expr::Select(_, inner)
                | Expr::Index(_, inner)
                | Expr::Filter(_, inner)
                | Expr::Prob(_, inner) => {
                    write_node(f, inner, &next_prefix, true)?;
                }
                Expr::Cross(_, lhs, rhs) => {
                    writeln!(f, "{next_prefix}├─ lhs")?;
                    write_node(f, lhs, &format!("{next_prefix}│  "), true)?;
                    writeln!(f, "{next_prefix}└─ rhs")?;
                    write_node(f, rhs, &format!("{next_prefix}   "), true)?;
                }
                Expr::Mut(_) | Expr::NoOp => {}
                Expr::Fused(fused) => match fused {
                    FusedExpr::Mutate(_, f) => {
                        // writeln!(f, "{next_prefix}└─ func")?;
                        // write_node(f, f, &format!("{next_prefix}   "), true)?;
                    }
                    FusedExpr::None => {}
                },
            }

            Ok(())
        }

        write_node(f, self.0, "", true)
    }
}

// Handy convenience on Expr itself
impl<G: Gene> Expr<G> {
    /// Print a tree to stdout (convenience).
    pub fn dump_tree(&self) {
        println!("{}", ExprTree(self));
    }
}
