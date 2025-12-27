#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WindowBuffer<T> {
    buffer: Vec<T>,
    cap: usize,
    max: usize,
    start: usize,
    end: usize,
}

impl<T> WindowBuffer<T> {
    pub fn with_window(cap: usize) -> Self {
        assert!(cap > 0, "WindowBuffer capacity must be > 0");

        let max = cap * 2;
        Self {
            buffer: Vec::with_capacity(max),
            cap,
            max,
            start: 0,
            end: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, item: T) -> bool {
        let mut resized = false;
        // check if the buffer is at capacity
        if self.buffer.len() >= self.cap {
            // check if the buffer is full
            if self.buffer.len() >= self.max {
                if self.end >= self.max {
                    let (front, back) = self.buffer.split_at_mut(self.cap);
                    front.swap_with_slice(back);

                    self.start = 0;
                    self.end = self.cap;
                }
                self.buffer[self.end] = item;
            } else {
                self.buffer.push(item);
            }

            resized = true;

            self.start += 1;
            self.end += 1;
        } else {
            self.buffer.push(item);
            self.end += 1;
            if self.end - self.start > self.cap {
                self.start += 1;
            }
        }

        resized
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.len().saturating_sub(self.start)
    }

    #[inline]
    pub fn values(&self) -> &[T] {
        &self.buffer[self.start..self.end]
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.values().iter()
    }
}

impl<T: Clone> Clone for WindowBuffer<T> {
    fn clone(&self) -> Self {
        Self {
            buffer: self.buffer.clone(),
            cap: self.cap,
            max: self.max,
            start: self.start,
            end: self.end,
        }
    }
}

impl<T: PartialEq> PartialEq for WindowBuffer<T> {
    fn eq(&self, other: &Self) -> bool {
        self.values() == other.values()
    }
}

impl<T: Debug> Debug for WindowBuffer<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowBuffer")
            .field("values", &self.values())
            .field("capacity", &self.cap)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::WindowBuffer;

    #[test]
    fn ring_buffer_works() {
        let mut buffer = WindowBuffer::with_window(5);
        for i in 0..20 {
            buffer.push(i);
            println!(
                "Added value {}, buffer len: {:?} -> {:?} ",
                i,
                buffer.len(),
                buffer.values(),
            );
        }
    }
}
