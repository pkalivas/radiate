use super::{Motion, nav::DashboardTab};
use radiate_engines::{phenotype::PhenotypeId, species::SpeciesId};
use radiate_utils::SmallStr;
use ratatui::widgets::{ScrollbarState, TableState};

pub struct TableStates {
    pub time: AppTableState<SmallStr>,
    pub stats: AppTableState<SmallStr>,
    pub dist: AppTableState<SmallStr>,
    pub species: AppTableState<SpeciesId>,
    pub population: AppTableState<PhenotypeId>,
    pub log: AppTableState<usize>,
    pub events: AppTableState<usize>,
}

impl TableStates {
    /// The list on `tab`. Log and Front share one.
    pub fn selection_mut(&mut self, tab: DashboardTab) -> &mut Selection {
        match tab {
            DashboardTab::Stats => &mut self.stats.selection,
            DashboardTab::Time => &mut self.time.selection,
            DashboardTab::Distribution => &mut self.dist.selection,
            DashboardTab::Events => &mut self.events.selection,
            DashboardTab::Population => &mut self.population.selection,
            DashboardTab::Species => &mut self.species.selection,
            DashboardTab::Log | DashboardTab::Front => &mut self.log.selection,
        }
    }

    pub fn selected_metric(&self, tab: DashboardTab) -> Option<&str> {
        match tab {
            DashboardTab::Time => self.time.selected_value.as_deref(),
            DashboardTab::Stats => self.stats.selected_value.as_deref(),
            DashboardTab::Distribution => self.dist.selected_value.as_deref(),
            DashboardTab::Population
            | DashboardTab::Species
            | DashboardTab::Log
            | DashboardTab::Front
            | DashboardTab::Events => None,
        }
    }
}

impl Default for TableStates {
    fn default() -> Self {
        Self {
            time: AppTableState::new(),
            stats: AppTableState::new(),
            dist: AppTableState::new(),
            species: AppTableState::new(),
            population: AppTableState::new(),
            log: AppTableState::new(),
            events: AppTableState::new(),
        }
    }
}

/// Where a table's cursor is. None of it depends on what the rows hold, so
/// every table moves the same way.
#[derive(Default)]
pub struct Selection {
    pub state: TableState,
    pub scroll_bar: Option<ScrollbarState>,
    pub selected_row: usize,
    pub row_count: usize,
    pub visible_rows: usize,
}

impl Selection {
    /// Up and Down wrap around the ends; paging and jumps stop at them.
    pub fn apply(&mut self, motion: Motion) {
        if self.row_count == 0 {
            return;
        }

        let last = self.row_count - 1;
        let current = self.state.selected();
        let page = self.visible_rows.max(1);

        let target = match motion {
            Motion::Down => current.map_or(0, |i| if i >= last { 0 } else { i + 1 }),
            Motion::Up => current.map_or(last, |i| if i == 0 { last } else { i - 1 }),
            Motion::PageDown => current.unwrap_or(0) + page,
            Motion::PageUp => current.unwrap_or(0).saturating_sub(page),
            Motion::Top => 0,
            Motion::Bottom => last,
        };

        self.select(target);
    }

    fn select(&mut self, row: usize) {
        let row = row.min(self.row_count.saturating_sub(1));
        self.state.select(Some(row));
        self.selected_row = row;
        self.scroll_bar = self.scroll_bar.take().map(|sb| sb.position(row));
    }
}

pub struct AppTableState<T> {
    pub selection: Selection,
    pub selected_value: Option<T>,
}

impl<T> AppTableState<T> {
    pub fn new() -> Self {
        Self {
            selection: Selection::default(),
            selected_value: None,
        }
    }

    /// Called on render with the rows about to be drawn: keeps the cursor in
    /// range and records which value it's on.
    pub fn update_rows<K, F>(&mut self, items: &[K], func: F)
    where
        F: Fn(&K) -> T,
    {
        let sel = &mut self.selection;
        let previous = sel.row_count;
        sel.row_count = items.len();

        if (previous == 0 && sel.row_count > 0)
            || (sel.selected_row < previous && sel.selected_row >= sel.row_count)
        {
            sel.selected_row = 0;
            sel.state.select(Some(0));
        }

        if sel.selected_row >= sel.row_count && sel.row_count > 0 {
            sel.selected_row = sel.row_count - 1;
            sel.state.select(Some(sel.selected_row));
        }

        self.selected_value = items.get(sel.selected_row).map(&func);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn selection(rows: usize, visible: usize) -> Selection {
        Selection {
            row_count: rows,
            visible_rows: visible,
            ..Default::default()
        }
    }

    #[test]
    fn up_and_down_wrap() {
        let mut s = selection(3, 10);
        s.apply(Motion::Up);
        assert_eq!(s.selected_row, 2);
        s.apply(Motion::Down);
        assert_eq!(s.selected_row, 0);
    }

    #[test]
    fn paging_stops_at_the_ends() {
        let mut s = selection(25, 10);
        s.apply(Motion::PageDown);
        s.apply(Motion::PageDown);
        s.apply(Motion::PageDown);
        assert_eq!(s.selected_row, 24);
        s.apply(Motion::PageUp);
        assert_eq!(s.selected_row, 14);
        s.apply(Motion::Top);
        assert_eq!(s.selected_row, 0);
    }

    #[test]
    fn an_empty_table_ignores_motion() {
        let mut s = selection(0, 10);
        s.apply(Motion::Bottom);
        assert_eq!(s.state.selected(), None);
    }
}
