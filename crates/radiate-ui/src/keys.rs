//! Every key the UI responds to. [`BINDINGS`] is read both to dispatch a key
//! press and to draw the help overlay, so the two can't disagree.

use crate::state::{Dir, Motion, UiMode};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Something a key press can do. What it acts on (which list, which chart)
/// is decided by the current tab when it's applied.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Quit,
    ToggleHelp,
    TogglePause,
    Step,

    /// Move the cursor in the active tab's list.
    Select(Motion),
    Tab(Dir),
    /// Cycle the active tab's view: chart views, or chromosomes on Population.
    Cycle(Dir),
    /// Scroll the chromosome shown on Population.
    Inspect(Motion),

    ToggleModal,
    ToggleLogScale,
    CycleHistory,
    ObjectivePage(Dir),
    ObjectivePairs(Dir),
    /// Select the objective named by the digit pressed.
    Objective,

    OpenSearch,
    CloseSearch,
    CancelSearch,
    ClearFilter,
    /// Type the character pressed into the search box.
    Type,
    Backspace,
}

/// A key, or a class of keys, a binding answers to.
#[derive(Clone, Copy, Debug)]
pub enum Key {
    /// This key, with or without modifiers (Shift arrives with capitals).
    Code(KeyCode),
    /// This letter with Ctrl held.
    Ctrl(char),
    /// Any digit.
    Digit,
    /// Any character.
    Char,
}

impl Key {
    fn matches(self, event: &KeyEvent) -> bool {
        match (self, event.code) {
            (Key::Code(code), pressed) => code == pressed,
            (Key::Ctrl(c), KeyCode::Char(pressed)) => {
                c == pressed && event.modifiers.contains(KeyModifiers::CONTROL)
            }
            (Key::Digit, KeyCode::Char(c)) => c.is_ascii_digit(),
            (Key::Char, KeyCode::Char(_)) => true,
            _ => false,
        }
    }

    pub fn label(self) -> String {
        match self {
            Key::Code(code) => match code {
                KeyCode::Char(c) => c.to_string(),
                KeyCode::Up => "↑".into(),
                KeyCode::Down => "↓".into(),
                KeyCode::Left => "←".into(),
                KeyCode::Right => "→".into(),
                KeyCode::PageUp => "PgUp".into(),
                KeyCode::PageDown => "PgDn".into(),
                KeyCode::Tab => "Tab".into(),
                KeyCode::BackTab => "S-Tab".into(),
                KeyCode::Enter => "Enter".into(),
                KeyCode::Esc => "Esc".into(),
                KeyCode::Home => "Home".into(),
                KeyCode::End => "End".into(),
                KeyCode::Backspace => "Bksp".into(),
                other => format!("{other:?}"),
            },
            Key::Ctrl(c) => format!("Ctrl-{}", c.to_ascii_uppercase()),
            Key::Digit => "0-9".into(),
            Key::Char => "text".into(),
        }
    }
}

pub struct Binding {
    pub modes: &'static [UiMode],
    pub keys: &'static [Key],
    pub action: Action,
    /// The help overlay's section. Bindings with no help text aren't listed.
    pub group: &'static str,
    pub help: &'static str,
}

impl Binding {
    pub fn label(&self) -> String {
        self.keys
            .iter()
            .map(|key| key.label())
            .collect::<Vec<_>>()
            .join(" / ")
    }
}

/// The action bound to `event` in `mode`. The first matching binding wins.
pub fn action(mode: UiMode, event: &KeyEvent) -> Option<Action> {
    BINDINGS
        .iter()
        .filter(|binding| binding.modes.contains(&mode))
        .find(|binding| binding.keys.iter().any(|key| key.matches(event)))
        .map(|binding| binding.action)
}

use Action::*;
use KeyCode::{BackTab, Down, End, Enter, Esc, Home, Left, PageDown, PageUp, Right, Up};
use UiMode::{Dashboard, Help, MetricModal, Search};

const fn c(ch: char) -> Key {
    Key::Code(KeyCode::Char(ch))
}

const fn k(code: KeyCode) -> Key {
    Key::Code(code)
}

const fn bind(
    modes: &'static [UiMode],
    keys: &'static [Key],
    action: Action,
    group: &'static str,
    help: &'static str,
) -> Binding {
    Binding {
        modes,
        keys,
        action,
        group,
        help,
    }
}

const GENERAL: &str = "General";
const NAVIGATION: &str = "Navigation";
const CHARTS: &str = "Charts / Objectives";
const POPULATION: &str = "Population";
const SEARCH: &str = "Search";

const DASHBOARD: &[UiMode] = &[Dashboard];
const ENGINE: &[UiMode] = &[Dashboard, MetricModal];

#[rustfmt::skip]
pub const BINDINGS: &[Binding] = &[
    bind(DASHBOARD, &[c('q')], Quit, GENERAL, "Quit"),
    bind(&[Dashboard, Help], &[c('?'), c('H')], ToggleHelp, GENERAL, "Toggle this help"),
    bind(&[Help], &[k(Esc)], ToggleHelp, GENERAL, ""),
    bind(ENGINE, &[c('p')], TogglePause, GENERAL, "Pause / resume the engine"),
    bind(ENGINE, &[c('n')], Step, GENERAL, "Step one epoch (stays paused)"),
    //
    bind(DASHBOARD, &[c('j'), k(Down)], Select(Motion::Down), NAVIGATION, "Move selection down"),
    bind(DASHBOARD, &[c('k'), k(Up)], Select(Motion::Up), NAVIGATION, "Move selection up"),
    bind(DASHBOARD, &[c('d'), k(PageDown)], Select(Motion::PageDown), NAVIGATION, "Page down"),
    bind(DASHBOARD, &[c('u'), k(PageUp)], Select(Motion::PageUp), NAVIGATION, "Page up"),
    bind(DASHBOARD, &[c('g'), k(Home)], Select(Motion::Top), NAVIGATION, "Jump to top"),
    bind(DASHBOARD, &[c('G'), k(End)], Select(Motion::Bottom), NAVIGATION, "Jump to bottom"),
    bind(DASHBOARD, &[c('h'), k(Left)], Tab(Dir::Prev), NAVIGATION, "Previous tab"),
    bind(DASHBOARD, &[c('l'), k(Right)], Tab(Dir::Next), NAVIGATION, "Next tab"),
    //
    bind(ENGINE, &[k(Enter)], ToggleModal, CHARTS, "Open / close the metric chart"),
    bind(&[MetricModal], &[k(Esc), c('q')], ToggleModal, CHARTS, ""),
    bind(ENGINE, &[k(KeyCode::Tab)], Cycle(Dir::Next), CHARTS, "Next chart view / chromosome"),
    bind(ENGINE, &[k(BackTab)], Cycle(Dir::Prev), CHARTS, "Previous chart view / chromosome"),
    bind(ENGINE, &[c('L')], ToggleLogScale, CHARTS, "Toggle log scale"),
    bind(ENGINE, &[c('z')], CycleHistory, CHARTS, "Show all / last 500 / last 100"),
    bind(&[MetricModal], &[k(Right)], Cycle(Dir::Next), CHARTS, ""),
    bind(&[MetricModal], &[k(Left)], Cycle(Dir::Prev), CHARTS, ""),
    bind(DASHBOARD, &[c(']')], ObjectivePage(Dir::Next), CHARTS, "Next objective-pair page"),
    bind(DASHBOARD, &[c('[')], ObjectivePage(Dir::Prev), CHARTS, "Previous objective-pair page"),
    bind(DASHBOARD, &[c('+')], ObjectivePairs(Dir::Next), CHARTS, "Show more objective pairs"),
    bind(DASHBOARD, &[c('-')], ObjectivePairs(Dir::Prev), CHARTS, "Show fewer objective pairs"),
    bind(DASHBOARD, &[Key::Digit], Objective, CHARTS, "Select an objective"),
    //
    bind(DASHBOARD, &[c('J')], Inspect(Motion::Down), POPULATION, "Scroll chromosome down"),
    bind(DASHBOARD, &[c('K')], Inspect(Motion::Up), POPULATION, "Scroll chromosome up"),
    bind(DASHBOARD, &[c('D')], Inspect(Motion::PageDown), POPULATION, "Page chromosome down"),
    bind(DASHBOARD, &[c('U')], Inspect(Motion::PageUp), POPULATION, "Page chromosome up"),
    //
    bind(DASHBOARD, &[c('/')], OpenSearch, SEARCH, "Search"),
    bind(DASHBOARD, &[k(Esc)], ClearFilter, SEARCH, "Clear the search filter"),
    bind(&[Search], &[k(Enter), k(Esc)], CloseSearch, SEARCH, "Finish searching"),
    bind(&[Search], &[c('/')], CloseSearch, SEARCH, ""),
    bind(&[Search], &[Key::Ctrl('c')], CancelSearch, SEARCH, "Cancel and clear the search"),
    bind(&[Search], &[k(KeyCode::Backspace)], Backspace, SEARCH, ""),
    bind(&[Search], &[Key::Char], Type, SEARCH, ""),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn the_same_key_means_different_things_per_mode() {
        let right = press(Right);
        assert_eq!(action(Dashboard, &right), Some(Tab(Dir::Next)));
        assert_eq!(action(MetricModal, &right), Some(Cycle(Dir::Next)));
        assert_eq!(action(Help, &right), None);
    }

    #[test]
    fn search_takes_text_but_not_ctrl_c() {
        assert_eq!(action(Search, &press(KeyCode::Char('q'))), Some(Type));
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(action(Search, &ctrl_c), Some(CancelSearch));
    }

    #[test]
    fn capitals_match_with_shift_held() {
        let shift_g = KeyEvent::new(KeyCode::Char('G'), KeyModifiers::SHIFT);
        assert_eq!(action(Dashboard, &shift_g), Some(Select(Motion::Bottom)));
    }

    #[test]
    fn digits_select_objectives() {
        assert_eq!(
            action(Dashboard, &press(KeyCode::Char('3'))),
            Some(Objective)
        );
    }
}
