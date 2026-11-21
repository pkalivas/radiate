use super::Motion;

/// Where the population tab's debug panel is looking: which chromosome of the
/// selected phenotype is shown and how far down it's scrolled.
#[derive(Default)]
pub struct InspectState {
    pub chromosome: usize,
    pub scroll: usize,
    /// Rows that fit in the panel body, set at render time.
    pub page: usize,
    max_scroll: usize,
    /// Population row this view belongs to; moving the selection starts over.
    row: usize,
}

impl InspectState {
    /// Called on render, before reading `chromosome`: resets the view when the
    /// population selection moved, and keeps the chromosome index in range.
    pub fn sync(&mut self, row: usize, chromosomes: usize) {
        if row != self.row {
            self.row = row;
            self.chromosome = 0;
            self.scroll = 0;
        }
        if self.chromosome >= chromosomes {
            self.chromosome = 0;
            self.scroll = 0;
        }
    }

    /// Called on render once the body's wrapped height is known.
    pub fn set_content_rows(&mut self, rows: usize, page: usize) {
        self.page = page;
        self.max_scroll = rows.saturating_sub(page);
        self.scroll = self.scroll.min(self.max_scroll);
    }

    pub fn next_chromosome(&mut self, chromosomes: usize) {
        if chromosomes > 0 {
            self.chromosome = (self.chromosome + 1) % chromosomes;
            self.scroll = 0;
        }
    }

    pub fn previous_chromosome(&mut self, chromosomes: usize) {
        if chromosomes > 0 {
            self.chromosome = (self.chromosome + chromosomes - 1) % chromosomes;
            self.scroll = 0;
        }
    }

    pub fn scroll(&mut self, motion: Motion) {
        match motion {
            Motion::Down => self.scroll_by(1),
            Motion::Up => self.scroll_by(-1),
            Motion::PageDown => self.scroll_page(1),
            Motion::PageUp => self.scroll_page(-1),
            Motion::Top => self.scroll = 0,
            Motion::Bottom => self.scroll = self.max_scroll,
        }
    }

    pub fn scroll_by(&mut self, delta: isize) {
        self.scroll = self
            .scroll
            .saturating_add_signed(delta)
            .min(self.max_scroll);
    }

    pub fn scroll_page(&mut self, pages: isize) {
        self.scroll_by(pages * self.page.max(1) as isize);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moving_the_selection_starts_over() {
        let mut s = InspectState::default();
        s.sync(0, 3);
        s.next_chromosome(3);
        s.set_content_rows(100, 10);
        s.scroll_by(5);

        s.sync(0, 3);
        assert_eq!((s.chromosome, s.scroll), (1, 5));

        s.sync(4, 3);
        assert_eq!((s.chromosome, s.scroll), (0, 0));
    }

    #[test]
    fn chromosomes_wrap_and_reset_scroll() {
        let mut s = InspectState::default();
        s.set_content_rows(100, 10);
        s.scroll_by(5);
        s.previous_chromosome(3);
        assert_eq!((s.chromosome, s.scroll), (2, 0));
        s.next_chromosome(3);
        assert_eq!(s.chromosome, 0);
    }

    #[test]
    fn scroll_stays_within_content() {
        let mut s = InspectState::default();
        s.set_content_rows(25, 10);
        s.scroll_by(-3);
        assert_eq!(s.scroll, 0);
        s.scroll_page(5);
        assert_eq!(s.scroll, 15);

        // Content shrank (e.g. a new generation) past the current offset.
        s.set_content_rows(12, 10);
        assert_eq!(s.scroll, 2);
    }
}
