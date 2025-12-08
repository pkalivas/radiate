#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TagKind {
    Selector,
    Alterer,
    Mutator,
    Crossover,
    Species,
    Failure,
    Age,
    Front,
    Derived,
    Other,
    Statistic,
    Time,
    Distribution,
}

impl TagKind {
    pub const COUNT: usize = 13;

    #[inline]
    pub fn from_index(idx: u8) -> Option<Self> {
        use TagKind::*;
        Some(match idx {
            0 => Selector,
            1 => Alterer,
            2 => Mutator,
            3 => Crossover,
            4 => Species,
            5 => Failure,
            6 => Age,
            7 => Front,
            8 => Derived,
            9 => Other,
            10 => Statistic,
            11 => Time,
            12 => Distribution,
            _ => return None,
        })
    }

    #[inline]
    pub fn bit(self) -> u16 {
        1 << (self as u8)
    }

    #[inline]
    pub fn as_str(&self) -> &'static str {
        use TagKind::*;
        match self {
            Selector => "Selector",
            Alterer => "Alterer",
            Mutator => "Mutator",
            Crossover => "Crossover",
            Species => "Species",
            Failure => "Failure",
            Age => "Age",
            Front => "Front",
            Derived => "Derived",
            Other => "Other",
            Statistic => "Statistic",
            Time => "Time",
            Distribution => "Distribution",
        }
    }
}

impl PartialOrd for TagKind {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some((*self as u8).cmp(&(*other as u8)))
    }
}

impl Ord for TagKind {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (*self as u8).cmp(&(*other as u8))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct TagMask(pub(crate) u16);

impl TagMask {
    #[inline]
    pub fn empty() -> Self {
        TagMask(0)
    }

    #[inline]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn has(self, kind: TagKind) -> bool {
        self.0 & kind.bit() != 0
    }

    #[inline]
    pub fn insert(&mut self, kind: TagKind) {
        self.0 |= kind.bit();
    }

    #[inline]
    pub fn union(self, other: TagMask) -> TagMask {
        TagMask(self.0 | other.0)
    }

    #[inline]
    pub fn iter(self) -> TagMaskIter {
        TagMaskIter { bits: self.0 }
    }
}

impl From<u16> for TagMask {
    #[inline]
    fn from(bits: u16) -> Self {
        TagMask(bits)
    }
}

pub struct TagMaskIter {
    bits: u16,
}

impl Iterator for TagMaskIter {
    type Item = TagKind;

    fn next(&mut self) -> Option<Self::Item> {
        if self.bits == 0 {
            return None;
        }

        let tz = self.bits.trailing_zeros() as u8;
        self.bits &= self.bits - 1; // clear lowest set bit

        TagKind::from_index(tz)
    }
}

impl IntoIterator for TagMask {
    type Item = TagKind;
    type IntoIter = TagMaskIter;

    fn into_iter(self) -> Self::IntoIter {
        TagMaskIter { bits: self.0 }
    }
}
