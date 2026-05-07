use crate::error::RadiateResult;

pub trait StreamingIterator {
    /// The type of the elements being iterated over.
    type Item: ?Sized;

    /// Advances the iterator to the next element.
    ///
    /// Iterators start just before the first element, so this should be called before `get`.
    ///
    /// The behavior of calling this method after the end of the iterator has been reached is
    /// unspecified.
    fn advance(&mut self) -> RadiateResult<()>;

    /// Returns a reference to the current element of the iterator.
    ///
    /// The behavior of calling this method before `advance` has been called is unspecified.
    fn get(&self) -> Option<&Self::Item>;

    fn source(&self) -> Option<&Self::Item> {
        self.get()
    }

    /// Advances the iterator and returns the next value.
    ///
    /// The behavior of calling this method after the end of the iterator has been reached is
    /// unspecified.
    ///
    /// The default implementation simply calls `advance` followed by `get`.
    #[inline]
    fn next(&mut self) -> Option<&Self::Item> {
        self.advance().ok()?;
        (*self).get()
    }

    fn take(self, n: usize) -> Take<Self>
    where
        Self: Sized,
    {
        Take {
            it: self,
            n,
            done: false,
        }
    }

    /// Call a closure on each element, passing the element on.
    /// The closure is called upon calls to `advance` or `advance_back`, and exactly once per element
    /// regardless of how many times (if any) `get` is called.
    #[inline]
    fn inspect<F>(self, f: F) -> Inspect<Self, F>
    where
        F: FnMut(&Self::Item),
        Self: Sized,
    {
        Inspect { it: self, f }
    }

    /// Creates an iterator which transforms elements of this iterator by passing them to a closure.
    #[inline]
    fn map<B, F>(self, f: F) -> Map<Self, B, F>
    where
        Self: Sized,
        F: FnMut(&Self::Item) -> B,
    {
        Map {
            it: self,
            f,
            item: None,
        }
    }

    #[inline]
    fn last<B>(self) -> B
    where
        Self: Sized,
        B: FromStreamingIterator<Self::Item>,
    {
        B::from_streaming_iterator(self)
    }
}

pub trait FromStreamingIterator<T: ?Sized> {
    fn from_streaming_iterator<I>(iter: I) -> Self
    where
        I: StreamingIterator<Item = T>;
}

#[derive(Clone, Debug)]
pub struct Take<I> {
    it: I,
    n: usize,
    done: bool,
}

impl<I> StreamingIterator for Take<I>
where
    I: StreamingIterator,
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) -> RadiateResult<()> {
        if self.n != 0 {
            self.it.advance()?;
            self.n -= 1;
        } else {
            self.done = true;
        }
        Ok(())
    }

    #[inline]
    fn get(&self) -> Option<&I::Item> {
        if self.done { None } else { self.it.get() }
    }

    fn source(&self) -> Option<&I::Item> {
        self.it.source()
    }
}

/// A streaming iterator that calls a function with element before yielding it.
#[derive(Debug)]
pub struct Inspect<I, F> {
    it: I,
    f: F,
}

impl<I, F> StreamingIterator for Inspect<I, F>
where
    I: StreamingIterator,
    F: FnMut(&I::Item),
{
    type Item = I::Item;

    #[inline]
    fn advance(&mut self) -> RadiateResult<()> {
        if let Some(item) = self.it.next() {
            (self.f)(item);
        }
        Ok(())
    }

    #[inline]
    fn get(&self) -> Option<&Self::Item> {
        self.it.get()
    }

    fn source(&self) -> Option<&Self::Item> {
        self.it.source()
    }
}

/// A streaming iterator which transforms the elements of a streaming iterator.
#[derive(Debug)]
pub struct Map<I, B, F> {
    it: I,
    f: F,
    item: Option<B>,
}

impl<I, B, F> StreamingIterator for Map<I, B, F>
where
    I: StreamingIterator,
    F: FnMut(&I::Item) -> B,
{
    type Item = B;

    #[inline]
    fn advance(&mut self) -> RadiateResult<()> {
        self.item = self.it.next().map(&mut self.f);
        Ok(())
    }

    #[inline]
    fn get(&self) -> Option<&B> {
        self.item.as_ref()
    }

    fn source(&self) -> Option<&Self::Item> {
        self.item.as_ref()
    }
}
