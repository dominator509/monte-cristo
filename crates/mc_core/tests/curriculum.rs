//! Integration tests for the curriculum system.
//!
//! Verifies rank progression, threshold boundaries, and interaction with
//! the calendar system (Study action).
//!
//! SPEC-001 section 7 is authoritative.

use mc_core::calendar::{CalendarAction, IfCalendar};
use mc_core::curriculum::{Curriculum, Discipline, RANK_THRESHOLDS};

#[test]
fn rank_threshold_values_match_spec() {
    // SPEC-001 section 7: thresholds at 1, 3, 7, 13, 21 cumulative months
    assert_eq!(RANK_THRESHOLDS, [0, 1, 3, 7, 13, 21]);
}

#[test]
fn all_disciplines_start_at_rank_0() {
    let c = Curriculum::new();
    for d in Discipline::ALL {
        assert_eq!(c.rank(d), 0, "expected rank 0 for discipline {:?}", d);
    }
}

#[test]
fn discipline_names_are_non_empty() {
    for d in Discipline::ALL {
        let name = d.name();
        assert!(!name.is_empty(), "discipline {:?} has empty name", d);
    }
}

// ---- Rank progression across all disciplines ----

#[test]
fn fencing_ranks() {
    let mut c = Curriculum::new();
    assert_eq!(c.rank(Discipline::Fencing), 0);

    c.add_months(Discipline::Fencing, RANK_THRESHOLDS[1]); // 1
    assert_eq!(c.rank(Discipline::Fencing), 1);

    c.add_months(Discipline::Fencing, RANK_THRESHOLDS[2] - RANK_THRESHOLDS[1]); // 2 more
    assert_eq!(c.rank(Discipline::Fencing), 2);

    c.add_months(Discipline::Fencing, RANK_THRESHOLDS[3] - RANK_THRESHOLDS[2]); // 4 more
    assert_eq!(c.rank(Discipline::Fencing), 3);

    c.add_months(Discipline::Fencing, RANK_THRESHOLDS[4] - RANK_THRESHOLDS[3]); // 6 more
    assert_eq!(c.rank(Discipline::Fencing), 4);

    c.add_months(Discipline::Fencing, RANK_THRESHOLDS[5] - RANK_THRESHOLDS[4]); // 8 more
    assert_eq!(c.rank(Discipline::Fencing), 5);
}

#[test]
fn chemistry_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::Chemistry, 21);
    assert_eq!(c.rank(Discipline::Chemistry), 5);
}

#[test]
fn natural_philosophy_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::NaturalPhilosophy, 13);
    assert_eq!(c.rank(Discipline::NaturalPhilosophy), 4);
}

#[test]
fn mathematics_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::Mathematics, 7);
    assert_eq!(c.rank(Discipline::Mathematics), 3);
}

#[test]
fn languages_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::Languages, 3);
    assert_eq!(c.rank(Discipline::Languages), 2);
}

#[test]
fn history_politics_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::HistoryPolitics, 1);
    assert_eq!(c.rank(Discipline::HistoryPolitics), 1);
}

#[test]
fn economics_ranks() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::Economics, 0);
    assert_eq!(c.rank(Discipline::Economics), 0);
}

// ---- Boundary tests ----

#[test]
fn rank_boundaries_exact_match() {
    let mut c = Curriculum::new();
    // Exactly at threshold → rank achieved
    c.add_months(Discipline::Fencing, 1);
    assert_eq!(c.rank(Discipline::Fencing), 1);

    c.add_months(Discipline::Fencing, 2); // now at 3
    assert_eq!(c.rank(Discipline::Fencing), 2);

    c.add_months(Discipline::Fencing, 4); // now at 7
    assert_eq!(c.rank(Discipline::Fencing), 3);
}

#[test]
fn rank_boundaries_just_below() {
    let mut c = Curriculum::new();
    // Just below rank 1 threshold
    c.add_months(Discipline::Fencing, 0);
    assert_eq!(c.rank(Discipline::Fencing), 0);

    // Add 0 months — should still be rank 0
    c.add_months(Discipline::Fencing, 0);
    assert_eq!(c.rank(Discipline::Fencing), 0);
}

#[test]
fn rank_5_is_maximum() {
    let mut c = Curriculum::new();
    c.add_months(Discipline::Fencing, 1000);
    assert_eq!(c.rank(Discipline::Fencing), 5);
    assert_eq!(c.months_for(Discipline::Fencing), 1000);
}

// ---- Calendar interaction ----

#[test]
fn calendar_study_propagates_to_curriculum() {
    let mut cal = IfCalendar::new();
    let mut cur = Curriculum::new();

    // Study Fencing for 5 months
    for _ in 0..5 {
        cal.advance(CalendarAction::Study(Discipline::Fencing), &mut cur);
    }

    assert_eq!(cur.months_for(Discipline::Fencing), 5);
    assert_eq!(cur.rank(Discipline::Fencing), 2); // 5 ≥ 3 → rank 2
}

#[test]
fn calendar_full_study_curriculum() {
    let mut cal = IfCalendar::new();
    let mut cur = Curriculum::new();

    // Study all 7 disciplines, each for 21 months = 147 months total
    // But we only have 168 months, so we can max out all 7
    let months_per = 21;
    for d in Discipline::ALL {
        for _ in 0..months_per {
            cal.advance(CalendarAction::Study(d), &mut cur);
        }
    }
    // 147 months used, 21 remaining

    for d in Discipline::ALL {
        assert_eq!(
            cur.rank(d),
            5,
            "expected rank 5 for {:?} after {} months",
            d,
            months_per
        );
    }
}

#[test]
fn calendar_dig_does_not_affect_curriculum() {
    let mut cal = IfCalendar::new();
    let mut cur = Curriculum::new();

    cal.advance(CalendarAction::Dig, &mut cur);

    for d in Discipline::ALL {
        assert_eq!(cur.months_for(d), 0);
    }
    assert_eq!(cal.dig_progress, 1);
}

#[test]
fn calendar_observe_does_not_affect_curriculum() {
    let mut cal = IfCalendar::new();
    let mut cur = Curriculum::new();

    cal.advance(CalendarAction::Observe, &mut cur);

    for d in Discipline::ALL {
        assert_eq!(cur.months_for(d), 0);
    }
    assert_eq!(cal.observe_knowledge, 1);
}

#[test]
fn calendar_endure_does_not_affect_curriculum() {
    let mut cal = IfCalendar::new();
    let mut cur = Curriculum::new();

    cal.advance(CalendarAction::Endure, &mut cur);

    for d in Discipline::ALL {
        assert_eq!(cur.months_for(d), 0);
    }
}

// ---- Season clock integration ----

#[test]
fn season_clock_fires_events() {
    use mc_core::season::{SeasonClock, SeasonEvent};
    let schedule = vec![
        SeasonEvent::new(0, "start"),
        SeasonEvent::new(12, "mid"),
        SeasonEvent::new(23, "end"),
    ];
    let mut clock = SeasonClock::new(schedule);
    let events = clock.advance(); // fire "start"
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "start");
}
