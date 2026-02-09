mod inline;
mod sorted;
mod value;
mod window;

pub use inline::{Buffer, BufferIntoIter, InlineBuffer};
pub use sorted::SortedBuffer;
pub use value::Value;
pub use window::WindowBuffer;

// use core::mem::MaybeUninit;
// use core::ptr;

// pub struct InlineBuffer<T, const N: usize> {
//     len: usize,
//     buf: [MaybeUninit<T>; N],
// }

// impl<T, const N: usize> InlineBuffer<T, N> {
//     #[inline]
//     pub fn new() -> Self {
//         Self {
//             len: 0,
//             buf: [const { MaybeUninit::uninit() }; N],
//         }
//     }

//     #[inline]
//     pub fn len(&self) -> usize {
//         self.len
//     }

//     #[inline]
//     pub fn is_empty(&self) -> bool {
//         self.len == 0
//     }

//     #[inline]
//     pub const fn capacity(&self) -> usize {
//         N
//     }

//     #[inline]
//     pub fn as_slice(&self) -> &[T] {
//         // SAFETY: first `len` entries are initialized
//         unsafe { core::slice::from_raw_parts(self.buf.as_ptr() as *const T, self.len) }
//     }

//     #[inline]
//     pub fn as_mut_slice(&mut self) -> &mut [T] {
//         // SAFETY: first `len` entries are initialized
//         unsafe { core::slice::from_raw_parts_mut(self.buf.as_mut_ptr() as *mut T, self.len) }
//     }

//     #[inline]
//     pub fn push(&mut self, value: T) -> Result<(), T> {
//         if self.len == N {
//             return Err(value);
//         }
//         unsafe { self.buf.get_unchecked_mut(self.len).write(value) };
//         self.len += 1;
//         Ok(())
//     }

//     #[inline]
//     pub fn pop(&mut self) -> Option<T> {
//         if self.len == 0 {
//             return None;
//         }
//         self.len -= 1;
//         Some(unsafe { self.buf.get_unchecked(self.len).assume_init_read() })
//     }

//     #[inline]
//     pub fn clear(&mut self) {
//         for i in 0..self.len {
//             unsafe { ptr::drop_in_place(self.buf.get_unchecked_mut(i).as_mut_ptr()) }
//         }
//         self.len = 0;
//     }
// }

// impl<T, const N: usize> Drop for InlineBuffer<T, N> {
//     fn drop(&mut self) {
//         for i in 0..self.len {
//             unsafe { ptr::drop_in_place(self.buf.get_unchecked_mut(i).as_mut_ptr()) }
//         }
//     }
// }

// impl<T: Clone, const N: usize> Clone for InlineBuffer<T, N> {
//     fn clone(&self) -> Self {
//         let mut new_buf = Self::new();
//         for item in self.as_slice() {
//             new_buf
//                 .push(item.clone())
//                 .unwrap_or_else(|_| panic!("InlineBuffer capacity of {} exceeded during clone", N));
//         }
//         new_buf
//     }
// }
