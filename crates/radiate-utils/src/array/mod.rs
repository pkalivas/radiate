mod error;
mod indices;
mod shape;
mod tensor;

pub use error::TensorError;
#[allow(dead_code)]
pub use shape::{Shape, Strides};
#[allow(dead_code)]
pub use tensor::Tensor;

// use super::{LayoutError, Shape, Strides};
// use core::ops::{Bound, RangeBounds};
// use std::slice;

// #[inline]
// fn is_contiguous_of(shape: &Shape, strides: &Strides) -> bool {
//     let expected = Strides::from(shape);
//     strides.as_slice() == expected.as_slice()
// }

// #[inline]
// fn is_contiguous_axis_of(shape: &Shape, strides: &Strides, axis: usize) -> bool {
//     let rank = shape.dimensions();
//     assert!(axis < rank, "axis out of bounds");
//     axis + 1 == rank && strides.stride_at(axis) == 1
// }

// #[inline]
// fn row_stride_of(strides: &Strides, axis: usize) -> usize {
//     strides.stride_at(axis)
// }

// #[inline]
// fn flat_index_unchecked_of(strides: &Strides, index: &[usize]) -> usize {
//     let mut flat = 0usize;
//     for (ax, &idx) in index.iter().enumerate() {
//         flat = flat.saturating_add(idx.saturating_mul(strides.stride_at(ax)));
//     }
//     flat
// }

// #[inline]
// fn checked_reshape_parts(len: usize, new_shape: Shape) -> Result<(Shape, Strides), LayoutError> {
//     let expected = new_shape
//         .try_size()
//         .ok_or_else(|| LayoutError::ShapeOverflow {
//             dims: new_shape.as_slice().to_vec(),
//         })?;

//     if expected != len {
//         return Err(LayoutError::LenMismatch { len, expected });
//     }

//     let strides = Strides::from(&new_shape);
//     Ok((new_shape, strides))
// }

// #[inline]
// fn permute_parts(shape: &Shape, strides: &Strides, perm: &[usize]) -> (Shape, Strides) {
//     let rank = shape.dimensions();
//     assert_eq!(perm.len(), rank, "perm length must match rank");

//     let mut seen = vec![false; rank];
//     for &p in perm {
//         assert!(p < rank, "perm out of bounds");
//         assert!(!seen[p], "perm contains duplicates");
//         seen[p] = true;
//     }

//     let new_dims = perm.iter().map(|&p| shape.dim_at(p)).collect::<Vec<_>>();
//     let new_strides = perm
//         .iter()
//         .map(|&p| strides.stride_at(p))
//         .collect::<Vec<_>>();

//     (Shape::from(new_dims), Strides::from(new_strides))
// }

// #[inline]
// fn try_broadcast_parts(
//     shape: &Shape,
//     strides: &Strides,
//     target: Shape,
// ) -> Result<(Shape, Strides), LayoutError> {
//     let src_rank = shape.dimensions();
//     let dst_rank = target.dimensions();
//     assert_eq!(src_rank, dst_rank, "broadcast ranks must match for now");

//     let mut new_strides = Vec::with_capacity(dst_rank);
//     for ax in 0..dst_rank {
//         let s = shape.dim_at(ax);
//         let t = target.dim_at(ax);
//         if s == t {
//             new_strides.push(strides.stride_at(ax));
//         } else if s == 1 && t >= 1 {
//             new_strides.push(0);
//         } else {
//             return Err(LayoutError::LenMismatch {
//                 len: s,
//                 expected: t,
//             });
//         }
//     }

//     Ok((target, Strides::from(new_strides)))
// }

// // ================================================================================================
// // View traits
// // ================================================================================================

// pub trait TensorViewLike<T> {
//     fn data(&self) -> &[T];
//     fn shape(&self) -> &Shape;
//     fn strides(&self) -> &Strides;

//     // ---- shape/stride basics ----

//     #[inline]
//     fn rank(&self) -> usize {
//         self.shape().dimensions()
//     }

//     #[inline]
//     fn axis_len(&self, axis: usize) -> usize {
//         self.shape().dim_at(axis)
//     }

//     #[inline]
//     fn len(&self) -> usize {
//         self.shape().size()
//     }

//     #[inline]
//     fn is_contiguous(&self) -> bool {
//         is_contiguous_of(self.shape(), self.strides())
//     }

//     #[inline]
//     fn is_contiguous_axis(&self, axis: usize) -> bool {
//         is_contiguous_axis_of(self.shape(), self.strides(), axis)
//     }

//     #[inline]
//     fn row_stride(&self, axis: usize) -> usize {
//         row_stride_of(self.strides(), axis)
//     }

//     // ---- fast slice access ----

//     /// Rank-1 zero-copy slice access for contiguous layouts.
//     #[inline]
//     fn as_slice_1d(&self) -> Option<&[T]> {
//         if self.rank() == 1 && self.is_contiguous() {
//             Some(self.data())
//         } else {
//             None
//         }
//     }

//     /// Returns raw parts for a contiguous 2D (rows, cols) matrix view.
//     #[inline]
//     fn as_2d_contiguous(&self) -> Option<(&[T], usize, usize)> {
//         if self.rank() != 2 || !self.is_contiguous_axis(1) {
//             return None;
//         }
//         Some((self.data(), self.axis_len(0), self.axis_len(1)))
//     }

//     /// Fast row access for a 2-D contiguous row-major view.
//     #[inline]
//     fn row(&self, r: usize) -> Option<&[T]> {
//         if self.rank() != 2 || !self.is_contiguous_axis(1) {
//             return None;
//         }

//         let rows = self.axis_len(0);
//         if r >= rows {
//             return None;
//         }

//         let cols = self.axis_len(1);
//         let start = r.saturating_mul(self.strides().stride_at(0));
//         Some(&self.data()[start..start + cols])
//     }

//     // ---- indexing ----

//     #[inline]
//     fn flat_index(&self, index: &[usize]) -> usize {
//         super::shape::flat_index_of(self.shape(), self.strides(), index)
//     }

//     #[inline]
//     fn get_nd(&self, index: &[usize]) -> &T {
//         let flat = self.flat_index(index);
//         &self.data()[flat]
//     }

//     #[inline]
//     unsafe fn get_flat_unchecked(&self, idx: usize) -> &T {
//         unsafe { self.data().get_unchecked(idx) }
//     }

//     #[inline]
//     unsafe fn get_nd_unchecked(&self, index: &[usize]) -> &T {
//         let flat = flat_index_unchecked_of(self.strides(), index);
//         unsafe { self.data().get_unchecked(flat) }
//     }

//     // ---- view transforms (owned wrappers) ----

//     #[inline]
//     fn reshape_owned(
//         &self,
//         new_shape: impl Into<Shape>,
//     ) -> Result<TensorOwnedView<'_, T>, LayoutError> {
//         let new_shape = new_shape.into();
//         let (shape, strides) = checked_reshape_parts(self.len(), new_shape)?;
//         Ok(TensorOwnedView {
//             data: self.data(),
//             shape,
//             strides,
//         })
//     }

//     /// Slice last axis by range when last axis is contiguous. Returns an owned view.
//     fn slice_last_axis_owned(
//         &self,
//         base: &[usize],
//         range: impl RangeBounds<usize>,
//     ) -> Result<TensorOwnedView<'_, T>, LayoutError> {
//         let rank = self.rank();
//         if base.len() != rank {
//             return Err(LayoutError::RankMismatch {
//                 got: base.len(),
//                 expected: rank,
//             });
//         }

//         let axis = rank.saturating_sub(1);
//         if !self.is_contiguous_axis(axis) {
//             return Err(LayoutError::NonContiguousRow { axis });
//         }

//         let len = self.axis_len(axis);
//         let start = match range.start_bound() {
//             Bound::Unbounded => 0,
//             Bound::Included(&s) => s,
//             Bound::Excluded(&s) => s + 1,
//         };
//         let end = match range.end_bound() {
//             Bound::Unbounded => len,
//             Bound::Included(&e) => e + 1,
//             Bound::Excluded(&e) => e,
//         };
//         assert!(start <= end && end <= len, "range out of bounds");

//         let start_flat = super::shape::row_start_flat_of(self.shape(), self.strides(), base, axis);
//         let offset = start_flat + start * self.strides().stride_at(axis);
//         let slice = &self.data()[offset..offset + (end - start)];

//         let mut new_dims = self.shape().as_slice().to_vec();
//         new_dims[axis] = end - start;

//         Ok(TensorOwnedView {
//             data: slice,
//             shape: Shape::from(new_dims),
//             strides: Strides::from(self.strides().as_slice().to_vec()),
//         })
//     }
// }

// pub trait TensorViewLikeMut<T>: TensorViewLike<T> {
//     fn data_mut(&mut self) -> &mut [T];

//     #[inline]
//     fn as_mut_slice(&mut self) -> &mut [T] {
//         self.data_mut()
//     }

//     #[inline]
//     fn get_nd_mut(&mut self, index: &[usize]) -> &mut T {
//         let flat = self.flat_index(index);
//         &mut self.data_mut()[flat]
//     }

//     #[inline]
//     unsafe fn get_flat_unchecked_mut(&mut self, idx: usize) -> &mut T {
//         unsafe { self.data_mut().get_unchecked_mut(idx) }
//     }

//     #[inline]
//     unsafe fn get_nd_unchecked_mut(&mut self, index: &[usize]) -> &mut T {
//         let flat = flat_index_unchecked_of(self.strides(), index);
//         unsafe { self.data_mut().get_unchecked_mut(flat) }
//     }

//     #[inline]
//     fn reshape_owned_mut(
//         &mut self,
//         new_shape: impl Into<Shape>,
//     ) -> Result<TensorOwnedViewMut<'_, T>, LayoutError> {
//         let new_shape = new_shape.into();
//         let (shape, strides) = checked_reshape_parts(self.len(), new_shape)?;
//         Ok(TensorOwnedViewMut {
//             data: self.data_mut(),
//             shape,
//             strides,
//         })
//     }

//     /// Rank-1 zero-copy mutable slice access for contiguous layouts.
//     ///
//     /// Note: this returns a slice tied to the borrow of `&mut self`, not `'a`.
//     #[inline]
//     fn as_mut_slice_1d(&mut self) -> Option<&mut [T]> {
//         if self.rank() == 1 && self.is_contiguous() {
//             Some(self.data_mut())
//         } else {
//             None
//         }
//     }
// }

// // ================================================================================================
// // View structs
// // ================================================================================================

// #[derive(Clone, Copy, Debug)]
// pub struct TensorView<'a, T> {
//     pub(crate) data: &'a [T],
//     pub(crate) shape: &'a Shape,
//     pub(crate) strides: &'a Strides,
// }

// #[derive(Debug)]
// pub struct TensorViewMut<'a, T> {
//     pub(crate) data: &'a mut [T],
//     pub(crate) shape: &'a Shape,
//     pub(crate) strides: &'a Strides,
// }

// #[derive(Clone, Debug)]
// pub struct TensorOwnedView<'a, T> {
//     pub(crate) data: &'a [T],
//     pub(crate) shape: Shape,
//     pub(crate) strides: Strides,
// }

// #[derive(Debug)]
// pub struct TensorOwnedViewMut<'a, T> {
//     pub(crate) data: &'a mut [T],
//     pub(crate) shape: Shape,
//     pub(crate) strides: Strides,
// }

// // ================================================================================================
// // Trait impls (the ONLY place these types define shared behavior)
// // ================================================================================================

// impl<'a, T> TensorViewLike<T> for TensorView<'a, T> {
//     #[inline]
//     fn data(&self) -> &[T] {
//         self.data
//     }
//     #[inline]
//     fn shape(&self) -> &Shape {
//         self.shape
//     }
//     #[inline]
//     fn strides(&self) -> &Strides {
//         self.strides
//     }
// }

// impl<'a, T> TensorViewLike<T> for TensorOwnedView<'a, T> {
//     #[inline]
//     fn data(&self) -> &[T] {
//         self.data
//     }
//     #[inline]
//     fn shape(&self) -> &Shape {
//         &self.shape
//     }
//     #[inline]
//     fn strides(&self) -> &Strides {
//         &self.strides
//     }
// }

// impl<'a, T> TensorViewLike<T> for TensorViewMut<'a, T> {
//     #[inline]
//     fn data(&self) -> &[T] {
//         self.data
//     }
//     #[inline]
//     fn shape(&self) -> &Shape {
//         self.shape
//     }
//     #[inline]
//     fn strides(&self) -> &Strides {
//         self.strides
//     }
// }

// impl<'a, T> TensorViewLike<T> for TensorOwnedViewMut<'a, T> {
//     #[inline]
//     fn data(&self) -> &[T] {
//         self.data
//     }
//     #[inline]
//     fn shape(&self) -> &Shape {
//         &self.shape
//     }
//     #[inline]
//     fn strides(&self) -> &Strides {
//         &self.strides
//     }
// }

// impl<'a, T> TensorViewLikeMut<T> for TensorViewMut<'a, T> {
//     #[inline]
//     fn data_mut(&mut self) -> &mut [T] {
//         self.data
//     }
// }

// impl<'a, T> TensorViewLikeMut<T> for TensorOwnedViewMut<'a, T> {
//     #[inline]
//     fn data_mut(&mut self) -> &mut [T] {
//         self.data
//     }
// }

// // ================================================================================================
// // Type-specific constructors / unsafe raw parts
// // (keep these here; everything else should live on the traits)
// // ================================================================================================

// impl<'a, T> TensorView<'a, T> {
//     /// Unsafe: build an owned view from raw parts without validation.
//     pub unsafe fn from_raw_parts(
//         data: *const T,
//         len: usize,
//         shape: impl Into<Shape>,
//         strides: impl Into<Strides>,
//     ) -> TensorOwnedView<'a, T> {
//         let shape = shape.into();
//         let strides = strides.into();
//         let data = unsafe { slice::from_raw_parts(data, len) };
//         TensorOwnedView {
//             data,
//             shape,
//             strides,
//         }
//     }
// }

// impl<'a, T> TensorViewMut<'a, T> {
//     /// Unsafe: build a mutable owned view from raw parts without validation.
//     pub unsafe fn from_raw_parts_mut(
//         data: *mut T,
//         len: usize,
//         shape: impl Into<Shape>,
//         strides: impl Into<Strides>,
//     ) -> TensorOwnedViewMut<'a, T> {
//         let shape = shape.into();
//         let strides = strides.into();
//         let data = unsafe { slice::from_raw_parts_mut(data, len) };
//         TensorOwnedViewMut {
//             data,
//             shape,
//             strides,
//         }
//     }
// }

// // ================================================================================================
// // Owned-view-only ops (need ownership of shape/strides)
// // ================================================================================================

// impl<'a, T> TensorOwnedView<'a, T> {
//     /// Permute axes; preserves non-row-major strides by permuting strides.
//     pub fn permute_owned(&self, perm: &[usize]) -> Result<TensorOwnedView<'a, T>, LayoutError> {
//         let (shape, strides) = permute_parts(&self.shape, &self.strides, perm);
//         Ok(TensorOwnedView {
//             data: self.data,
//             shape,
//             strides,
//         })
//     }

//     /// Try broadcast to target shape; dims 1 can expand (stride->0).
//     pub fn try_broadcast_owned(
//         &self,
//         target: impl Into<Shape>,
//     ) -> Result<TensorOwnedView<'a, T>, LayoutError> {
//         let target = target.into();
//         let (shape, strides) = try_broadcast_parts(&self.shape, &self.strides, target)?;
//         Ok(TensorOwnedView {
//             data: self.data,
//             shape,
//             strides,
//         })
//     }
// }

// impl<'a, T> TensorOwnedViewMut<'a, T> {
//     pub fn permute_owned(
//         &mut self,
//         perm: &[usize],
//     ) -> Result<TensorOwnedViewMut<'_, T>, LayoutError> {
//         let (shape, strides) = permute_parts(&self.shape, &self.strides, perm);
//         Ok(TensorOwnedViewMut {
//             data: self.data,
//             shape,
//             strides,
//         })
//     }

//     pub fn try_broadcast_owned(
//         &mut self,
//         target: impl Into<Shape>,
//     ) -> Result<TensorOwnedViewMut<'_, T>, LayoutError> {
//         let target = target.into();
//         let (shape, strides) = try_broadcast_parts(&self.shape, &self.strides, target)?;
//         Ok(TensorOwnedViewMut {
//             data: self.data,
//             shape,
//             strides,
//         })
//     }
// }

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use crate::{Tensor, array::view::TensorViewLike};

//     #[test]
//     fn view_reports_contiguity_for_standard_layout() {
//         let t = Tensor::from_shape_fn((2, 3), |i| i as i32);
//         let v = t.view();

//         assert!(v.is_contiguous());
//         assert!(v.is_contiguous_axis(1));
//         assert!(!v.is_contiguous_axis(0));

//         assert!(v.as_slice_1d().is_none());
//     }

//     #[test]
//     fn view_reshape_owned_validates_len() {
//         let t = Tensor::from_shape_fn((2, 3), |i| i as i32);
//         let v = t.view();

//         let r = v.reshape_owned((3, 2)).unwrap();
//         assert_eq!(r.shape().as_slice(), &[3, 2]);
//         assert_eq!(r.get_nd(&[0, 0]), &0);
//         assert_eq!(r.get_nd(&[2, 1]), &5);

//         assert!(v.reshape_owned((4, 2)).is_err());
//     }

//     #[test]
//     fn slice_last_axis_returns_expected_window() {
//         let t = Tensor::from_shape_fn((2, 5), |i| i as i32);
//         println!("{:?}", t);
//         let v = t.view();

//         let s = v.slice_last_axis_owned(&[1, 0], 1..4).unwrap();

//         assert_eq!(s.shape().as_slice(), &[2, 3]);
//         assert_eq!(s.data(), &[6, 7, 8]);
//     }

//     #[test]
//     fn owned_view_permute_breaks_contiguity() {
//         let t = Tensor::from_shape_fn((2, 3), |i| i as i32);
//         let binding = t.view();
//         let owned = binding.reshape_owned((2, 3)).unwrap();

//         let p = owned.permute_owned(&[1, 0]).unwrap();
//         assert_eq!(p.shape().as_slice(), &[3, 2]);

//         let expected = Strides::from(p.shape());
//         assert_ne!(p.strides().as_slice(), expected.as_slice());
//     }

//     #[test]
//     fn owned_view_broadcast_sets_zero_stride_and_repeats_values() {
//         let t = Tensor::from_shape_fn((2, 1), |i| (i as i32) + 10);
//         let binding = t.view();
//         let owned = binding.reshape_owned((2, 1)).unwrap();
//         let b = owned.try_broadcast_owned((2, 3)).unwrap();

//         assert_eq!(b.shape().as_slice(), &[2, 3]);

//         assert_eq!(*b.get_nd(&[0, 0]), 10);
//         assert_eq!(*b.get_nd(&[0, 2]), 10);
//         assert_eq!(*b.get_nd(&[1, 1]), 11);
//         assert_eq!(*b.get_nd(&[1, 2]), 11);

//         assert_eq!(b.strides().stride_at(1), 0);
//     }

//     #[test]
//     fn view_mut_as_mut_slice_1d_is_zero_copy_and_writes_through() {
//         let mut t = Tensor::from_shape_fn(5usize, |i| i as i32);

//         {
//             let mut vm = t.view_mut();
//             let s = vm.as_mut_slice_1d().expect("should be rank-1 contiguous");
//             s[2] = 99;
//             s[4] = 123;
//         }

//         assert_eq!(t.data(), &[0, 1, 99, 3, 123]);
//     }
// }
