//! Status flag classification for entry types and freshness.
//!
//! This module computes machine-readable flags for an entry based on its
//! frontmatter (task status, event presence, timestamps). Display rendering
//! (emoji, nerd-font glyphs, initials) is handled via [`EntryFlag`] methods.
//!
//! [`EntryFlag::Stale`] and [`EntryFlag::Hidden`] are orthogonal to the
//! type/freshness slots: a task can be both `in_progress` *and* `stale`, or a
//! note can be `hidden`. They are appended after the slot-1/slot-2 flags.

use chrono::{Duration, Local, NaiveDateTime};

use crate::entry::{EventMetaView, TaskMetaView};

/// A computed flag describing an entry's type or freshness state.
///
/// Serializes to its string representation via [`EntryFlag::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryFlag {
    // Freshness / urgency (slot 1)
    Overdue,
    New,
    Updated,
    // Entry type (slot 2)
    Event,
    /// Past event whose `end` timestamp is before the current time.
    EventClosed,
    Done,
    Cancelled,
    InProgress,
    Archived,
    Open,
    Note,
    // Orthogonal flags (appended after the slot-1/slot-2 flags)
    /// Incomplete task (`closed_at` absent) whose `updated_at` is older than the
    /// configured `stale_after_days` threshold.
    Stale,
    /// Entry explicitly marked `hidden: true` in its frontmatter.
    Hidden,
}

impl EntryFlag {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overdue     => "overdue",
            Self::New         => "new",
            Self::Updated     => "updated",
            Self::Event       => "event",
            Self::EventClosed => "event_closed",
            Self::Done        => "done",
            Self::Cancelled   => "cancelled",
            Self::InProgress  => "in_progress",
            Self::Archived    => "archived",
            Self::Open        => "open",
            Self::Note        => "note",
            Self::Stale       => "stale",
            Self::Hidden      => "hidden",
        }
    }

    pub fn to_emoji(self) -> &'static str {
        match self {
            Self::Overdue     => "⏰",
            Self::New         => "🆕",
            Self::Updated     => "✏️",
            Self::Event       => "📅",
            Self::EventClosed => "🗓️",
            Self::Done        => "✅",
            Self::Cancelled   => "❌",
            Self::InProgress  => "🔄",
            Self::Archived    => "📦",
            Self::Open        => "⬜",
            Self::Note        => "📝",
            Self::Stale       => "🕰️",
            Self::Hidden      => "🙈",
        }
    }

    pub fn to_nerd(self) -> &'static str {
        match self {
            Self::Overdue     => "󱦟",
            Self::New         => "󰐕",
            Self::Updated     => "󰏫",
            Self::Event       => "󰃭",
            Self::EventClosed => "󰄻",
            Self::Done        => "󰄲",
            Self::Cancelled   => "󰜺",
            Self::InProgress  => "󰔛",
            Self::Archived    => "󰀼",
            Self::Open        => "󰄱",
            Self::Note        => "󰈙",
            Self::Stale       => "󰔚",
            Self::Hidden      => "󰈈",
        }
    }

    pub fn to_initial(self) -> char {
        match self {
            Self::Overdue     => '!',
            Self::New         => '+',
            Self::Updated     => '~',
            Self::Event       => 'E',
            Self::EventClosed => 'e',
            Self::Done        => 'D',
            Self::Cancelled   => 'C',
            Self::InProgress  => 'I',
            Self::Archived    => 'A',
            Self::Open        => 'O',
            Self::Note        => 'N',
            Self::Stale       => 'S',
            Self::Hidden      => 'H',
        }
    }
}

impl serde::Serialize for EntryFlag {
    fn serialize<S: serde::Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.as_str())
    }
}

/// Returns the canonical flag string for a task status string.
///
/// Conventional statuses: `open`, `in_progress`, `done`, `cancelled`, `archived`.
/// Any unrecognised status is treated as `open`.
pub fn task_status_label(status: &str) -> &'static str {
    match status {
        "done" | "completed"     => "done",
        "cancelled" | "canceled" => "cancelled",
        "in_progress" | "wip"   => "in_progress",
        "archived"               => "archived",
        _                        => "open",
    }
}

/// Returns `true` when a task is *stale*: incomplete (`closed_at` absent) and
/// not updated for at least `stale_after`.
///
/// Events and notes (no task) are never stale, and a task with `closed_at` set
/// (done/cancelled/archived) is never stale regardless of age.
pub fn is_stale(task: Option<&TaskMetaView>, updated_at: NaiveDateTime, stale_after: Duration) -> bool {
    let incomplete_task = task.is_some_and(|t| t.closed_at.is_none());
    incomplete_task && (Local::now().naive_local() - updated_at) >= stale_after
}

/// Returns the computed [`EntryFlag`]s for an entry.
///
/// Slot 1 (urgency/freshness): `Overdue`, `New` (created <24 h), `Updated` (<24 h), absent otherwise.
/// Slot 2 (entry type): `Event` / `EventClosed` (past event), task status flag, or `Note`.
///
/// Appended afterwards, orthogonal to the slots: `Stale` (an incomplete task whose
/// `updated_at` predates `now - stale_after`) and `Hidden` (`hidden: true`).
pub fn entry_flags(
    task: Option<&TaskMetaView>,
    event: Option<&EventMetaView>,
    hidden: bool,
    created_at: NaiveDateTime,
    updated_at: NaiveDateTime,
    stale_after: Duration,
) -> Vec<EntryFlag> {
    let mut flags = Vec::new();

    let now = Local::now().naive_local();

    // Slot 1: overdue (highest priority) > created <24h > updated <24h
    let is_overdue = task.map_or(false, |t| {
        t.due.map_or(false, |due| due < now) && t.closed_at.is_none()
    });
    if is_overdue {
        flags.push(EntryFlag::Overdue);
    } else {
        let threshold = now - Duration::hours(24);
        if created_at >= threshold {
            flags.push(EntryFlag::New);
        } else if updated_at >= threshold {
            flags.push(EntryFlag::Updated);
        }
    }

    // Slot 2: entry type
    if let Some(ev) = event {
        if ev.end < now {
            flags.push(EntryFlag::EventClosed);
        } else {
            flags.push(EntryFlag::Event);
        }
    } else if let Some(task) = task {
        let flag = match task_status_label(&task.status) {
            "done"        => EntryFlag::Done,
            "cancelled"   => EntryFlag::Cancelled,
            "in_progress" => EntryFlag::InProgress,
            "archived"    => EntryFlag::Archived,
            _             => EntryFlag::Open,
        };
        flags.push(flag);
    } else {
        flags.push(EntryFlag::Note);
    }

    // Orthogonal flags, appended after the type slot.
    if is_stale(task, updated_at, stale_after) {
        flags.push(EntryFlag::Stale);
    }
    if hidden {
        flags.push(EntryFlag::Hidden);
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entry::{EventMetaView, TaskMetaView};

    /// An incomplete task (`closed_at` absent) with an optional start time.
    fn open_task() -> TaskMetaView {
        TaskMetaView { due: None, status: "open".into(), started_at: None, closed_at: None }
    }

    /// A closed task (`closed_at` present).
    fn closed_task(closed_at: chrono::NaiveDateTime) -> TaskMetaView {
        TaskMetaView {
            due: None,
            status: "done".into(),
            started_at: None,
            closed_at: Some(closed_at),
        }
    }

    fn event(start: chrono::NaiveDateTime, end: chrono::NaiveDateTime) -> EventMetaView {
        EventMetaView { start, end }
    }

    fn now() -> chrono::NaiveDateTime {
        Local::now().naive_local()
    }

    #[test]
    fn incomplete_task_exactly_at_threshold_is_stale() {
        // `updated_at` exactly `stale_after` old: `now - updated >= stale_after` holds.
        let stale_after = Duration::days(30);
        let updated_at = now() - stale_after;
        assert!(is_stale(Some(&open_task()), updated_at, stale_after));
    }

    #[test]
    fn incomplete_task_one_day_before_threshold_is_not_stale() {
        // One day short of the threshold -> not yet stale.
        let stale_after = Duration::days(30);
        let updated_at = now() - Duration::days(29);
        assert!(!is_stale(Some(&open_task()), updated_at, stale_after));
    }

    #[test]
    fn incomplete_task_beyond_threshold_is_stale() {
        let stale_after = Duration::days(30);
        let updated_at = now() - Duration::days(60);
        assert!(is_stale(Some(&open_task()), updated_at, stale_after));
    }

    #[test]
    fn closed_task_is_never_stale_regardless_of_age() {
        let stale_after = Duration::days(30);
        let updated_at = now() - Duration::days(365);
        assert!(!is_stale(Some(&closed_task(updated_at)), updated_at, stale_after));
    }

    #[test]
    fn event_and_note_are_never_stale() {
        let stale_after = Duration::days(30);
        let old = now() - Duration::days(365);
        // No task at all (event/note) -> never stale (is_stale only inspects the task).
        assert!(!is_stale(None, old, stale_after));
        assert!(!is_stale(None, now() - Duration::days(1), stale_after));
        let _ = event(old, old);
    }

    #[test]
    fn entry_flags_appends_stale_for_old_incomplete_task() {
        let stale_after = Duration::days(30);
        let updated_at = now() - Duration::days(40);
        let flags = entry_flags(
            Some(&open_task()),
            None,
            false,
            updated_at,
            updated_at,
            stale_after,
        );
        assert!(flags.contains(&EntryFlag::Stale));
        assert!(flags.contains(&EntryFlag::Open));
    }

    #[test]
    fn entry_flags_marks_hidden_true_as_hidden() {
        let stale_after = Duration::days(30);
        let now_dt = now();
        let flags = entry_flags(None, Some(&event(now_dt, now_dt)), true, now_dt, now_dt, stale_after);
        assert!(flags.contains(&EntryFlag::Hidden));
    }

    #[test]
    fn entry_flags_does_not_mark_hidden_false_or_absent_as_hidden() {
        let stale_after = Duration::days(30);
        let now_dt = now();
        let flags = entry_flags(None, None, false, now_dt, now_dt, stale_after);
        assert!(!flags.contains(&EntryFlag::Hidden));
        assert!(flags.contains(&EntryFlag::Note));
    }

    #[test]
    fn entry_flags_stale_and_hidden_are_both_appended_for_a_stale_hidden_task() {
        // A stale, explicitly-hidden open task carries both orthogonal flags.
        let stale_after = Duration::days(30);
        let updated_at = now() - Duration::days(40);
        let flags = entry_flags(
            Some(&open_task()),
            None,
            true,
            updated_at,
            updated_at,
            stale_after,
        );
        assert!(flags.contains(&EntryFlag::Stale));
        assert!(flags.contains(&EntryFlag::Hidden));
    }
}
