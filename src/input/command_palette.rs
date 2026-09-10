use std::borrow::Cow;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::config::Keybinds;

use super::keybind_help::keybind_help_groups;
use super::keybindings::KeybindAction;

#[derive(Clone, Debug)]
pub(crate) struct PaletteEntry {
    pub(crate) key: String,
    pub(crate) label: Cow<'static, str>,
    pub(crate) action: KeybindAction,
}

/// The command palette's inventory is the keybind help entries that carry an
/// action, so a new keybinding shows up in both surfaces with no second list
/// to maintain.
pub(crate) fn palette_entries(
    keybinds: &Keybinds,
    prefix: (KeyCode, KeyModifiers),
) -> Vec<PaletteEntry> {
    keybind_help_groups(keybinds, prefix)
        .into_iter()
        .flat_map(|(_, entries)| entries)
        .filter_map(|(key, label, action)| action.map(|action| PaletteEntry { key, label, action }))
        .collect()
}

/// Ranking by match quality rather than list order keeps a query that exactly
/// names one command from being answered by a longer command containing it.
fn match_rank(label: &str, query: &str) -> Option<u8> {
    let label = label.to_lowercase();
    if label == query {
        Some(0)
    } else if label.starts_with(query) {
        Some(1)
    } else if label.split_whitespace().any(|word| word.starts_with(query)) {
        Some(2)
    } else if label.contains(query) {
        Some(3)
    } else {
        None
    }
}

pub(crate) fn filter_palette_entries(entries: Vec<PaletteEntry>, query: &str) -> Vec<PaletteEntry> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return entries;
    }
    let mut ranked: Vec<(u8, usize, PaletteEntry)> = entries
        .into_iter()
        .enumerate()
        .filter_map(|(index, entry)| {
            match_rank(&entry.label, &query).map(|rank| (rank, index, entry))
        })
        .collect();
    ranked.sort_by_key(|(rank, index, _)| (*rank, *index));
    ranked.into_iter().map(|(_, _, entry)| entry).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries() -> Vec<PaletteEntry> {
        vec![
            PaletteEntry {
                key: "a".into(),
                label: Cow::Borrowed("new workspace"),
                action: KeybindAction::NewWorkspace,
            },
            PaletteEntry {
                key: "b".into(),
                label: Cow::Borrowed("new worktree"),
                action: KeybindAction::NewWorktree,
            },
            PaletteEntry {
                key: "c".into(),
                label: Cow::Borrowed("close workspace"),
                action: KeybindAction::CloseWorkspace,
            },
        ]
    }

    #[test]
    fn empty_query_keeps_original_order() {
        let filtered = filter_palette_entries(entries(), "");
        assert_eq!(
            filtered.iter().map(|e| e.action).collect::<Vec<_>>(),
            vec![
                KeybindAction::NewWorkspace,
                KeybindAction::NewWorktree,
                KeybindAction::CloseWorkspace,
            ]
        );
    }

    #[test]
    fn exact_match_outranks_a_longer_substring_match() {
        let filtered = filter_palette_entries(entries(), "new worktree");
        assert_eq!(filtered[0].action, KeybindAction::NewWorktree);
    }

    #[test]
    fn prefix_match_outranks_word_start_match() {
        // "new" is a prefix of "new workspace" and "new worktree", but only a
        // word-start match for "close workspace".
        let filtered = filter_palette_entries(entries(), "new");
        assert_eq!(filtered.len(), 2);
        assert!(filtered
            .iter()
            .all(|entry| entry.action != KeybindAction::CloseWorkspace));
    }

    #[test]
    fn word_start_match_outranks_mid_word_substring_match() {
        // "workspace" starts a word in both "new workspace" and "close
        // workspace"; ties break by original position.
        let filtered = filter_palette_entries(entries(), "workspace");
        assert_eq!(
            filtered.iter().map(|e| e.action).collect::<Vec<_>>(),
            vec![KeybindAction::NewWorkspace, KeybindAction::CloseWorkspace]
        );
    }

    #[test]
    fn no_match_returns_empty() {
        assert!(filter_palette_entries(entries(), "zzz").is_empty());
    }
}
