//! Type-name helper used by component traits' default `write` impls.
//!
//! The earlier "freeze" / `Frozen` infrastructure (a structured snapshot type
//! with a derive macro and supertrait) was removed in favor of a simpler
//! `fn write(&self, w: &mut dyn std::io::Write) -> std::io::Result<()>` method
//! on each component trait. This module retains the `short_type_name` helper
//! that those default impls use to render `type: <Name>` lines.

use std::fmt::Debug;

use radiate_error::radiate_err;

use crate::error::RadiateResult;

/// Walk the full type name and strip the module path of every segment, not
/// just the head. Segments are delimited by `<`, `>`, `,`, or space; each
/// segment becomes whatever follows its last `::`. So
/// `module::Foo<other::Bar<u8>>` renders as `Foo<Bar<u8>>`.
pub fn short_type_name<T: ?Sized>() -> String {
    let full = std::any::type_name::<T>();
    let mut out = String::with_capacity(full.len());
    let mut segment_start = 0usize;
    for (i, c) in full.char_indices() {
        if matches!(c, '<' | '>' | ',' | ' ') {
            out.push_str(strip_path(&full[segment_start..i]));
            out.push(c);
            segment_start = i + c.len_utf8();
        }
    }
    if segment_start < full.len() {
        out.push_str(strip_path(&full[segment_start..]));
    }
    out
}

fn strip_path(segment: &str) -> &str {
    match segment.rfind("::") {
        Some(idx) => &segment[idx + 2..],
        None => segment,
    }
}

pub trait Writer<T: ?Sized> {
    type Output;
    fn write(&self, item: &T) -> RadiateResult<Self::Output>;
}

pub struct DebugWriter;

impl<T: Debug + ?Sized> Writer<T> for DebugWriter {
    type Output = String;

    fn write(&self, item: &T) -> RadiateResult<String> {
        Ok(format!("{:?}", item))
    }
}
