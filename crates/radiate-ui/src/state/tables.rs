use super::nav::DashboardTab;
use radiate_engines::species::SpeciesId;
use radiate_utils::SmallStr;
use ratatui::widgets::{ScrollbarState, TableState};

pub struct TableStates {
    pub time: AppTableState<SmallStr>,
    pub stats: AppTableState<SmallStr>,
    pub dist: AppTableState<SmallStr>,
    pub species: AppTableState<SpeciesId>,
    pub log: AppTableState<usize>,
}

impl TableStates {
    pub fn move_down(&mut self, tab: DashboardTab) {
        match tab {
            DashboardTab::Stats => Self::scroll_down(&mut self.stats),
            DashboardTab::Time => Self::scroll_down(&mut self.time),
            DashboardTab::Distribution => Self::scroll_down(&mut self.dist),
            DashboardTab::Species => Self::scroll_down(&mut self.species),
            DashboardTab::Log | DashboardTab::Front => Self::scroll_down(&mut self.log),
        }
    }

    pub fn move_up(&mut self, tab: DashboardTab) {
        match tab {
            DashboardTab::Stats => Self::scroll_up(&mut self.stats),
            DashboardTab::Time => Self::scroll_up(&mut self.time),
            DashboardTab::Distribution => Self::scroll_up(&mut self.dist),
            DashboardTab::Species => Self::scroll_up(&mut self.species),
            DashboardTab::Log | DashboardTab::Front => Self::scroll_up(&mut self.log),
        }
    }

    pub fn selected_metric(&self, tab: DashboardTab) -> Option<&str> {
        match tab {
            DashboardTab::Time => self.time.selected_value.as_deref(),
            DashboardTab::Stats => self.stats.selected_value.as_deref(),
            DashboardTab::Distribution => self.dist.selected_value.as_deref(),
            DashboardTab::Species | DashboardTab::Log | DashboardTab::Front => None,
        }
    }

    fn scroll_down<T>(t: &mut AppTableState<T>) {
        if t.row_count == 0 {
            return;
        }
        if let Some(i) = t.state.selected() {
            let next = if i + 1 >= t.row_count { 0 } else { i + 1 };
            t.state.select(Some(next));
            t.selected_row = next;
            t.scroll_bar = t.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            t.state.select(Some(0));
            t.selected_row = 0;
            t.scroll_bar = t.scroll_bar.as_mut().map(|sb| sb.position(0));
        }
    }

    fn scroll_up<T>(t: &mut AppTableState<T>) {
        if t.row_count == 0 {
            return;
        }
        if let Some(i) = t.state.selected() {
            let next = if i == 0 { t.row_count - 1 } else { i - 1 };
            t.state.select(Some(next));
            t.selected_row = next;
            t.scroll_bar = t.scroll_bar.as_mut().map(|sb| sb.position(next));
        } else {
            let last = t.row_count - 1;
            t.state.select(Some(last));
            t.selected_row = last;
            t.scroll_bar = t.scroll_bar.as_mut().map(|sb| sb.position(last));
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
            log: AppTableState::new(),
        }
    }
}

pub struct AppTableState<T> {
    pub state: TableState,
    pub scroll_bar: Option<ScrollbarState>,
    pub selected_value: Option<T>,
    pub selected_row: usize,
    pub row_count: usize,
    pub prev_row_count: usize,
}

impl<T> AppTableState<T> {
    pub fn new() -> Self {
        Self {
            state: TableState::default(),
            scroll_bar: None,
            selected_row: 0,
            row_count: 0,
            prev_row_count: 0,
            selected_value: None,
        }
    }

    pub fn update_rows<K, F>(&mut self, items: &[K], func: F)
    where
        F: Fn(&K) -> T,
    {
        let current_len = items.len();
        self.prev_row_count = self.row_count;
        self.row_count = current_len;

        if (self.prev_row_count == 0 && self.row_count > 0)
            || (self.selected_row < self.prev_row_count && self.selected_row >= self.row_count)
        {
            self.selected_row = 0;
            self.state.select(Some(0));
        }

        if self.selected_row >= current_len && current_len > 0 {
            self.selected_row = current_len - 1;
            self.state.select(Some(self.selected_row));
        }

        self.selected_value = items.get(self.selected_row).map(&func);
    }
}
