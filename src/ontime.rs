use chrono::Timelike;
use serde::Serialize;

use crate::template::ExamApplication;

/// Default lead time (before the exam's `timeEnd`) for OnTime's warning and
/// danger timer states, clamped down for short exams so they never exceed
/// the exam's own duration.
const DEFAULT_WARNING_MS: u64 = 10 * 60 * 1000;
const DEFAULT_DANGER_MS: u64 = 2 * 60 * 1000;

/// One entry for OnTime's `ontime_batch_create_entries` MCP tool
/// (https://ontime.vini84200.cc/mcp). Only the fields this generator
/// actually sets are modeled; unset fields are omitted from the JSON so
/// OnTime falls back to its own defaults.
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OntimeEntry {
    #[serde(rename = "type")]
    pub kind: &'static str,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cue: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timer_type: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_action: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_warning: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_danger: Option<u64>,
}

fn milestone(cue: &str, title: String, note: String) -> OntimeEntry {
    OntimeEntry {
        kind: "milestone",
        title,
        cue: Some(cue.to_string()),
        note: Some(note),
        time_start: None,
        duration: None,
        timer_type: None,
        end_action: None,
        time_warning: None,
        time_danger: None,
    }
}

fn ms_from_midnight(time: chrono::NaiveTime) -> u64 {
    time.num_seconds_from_midnight() as u64 * 1000
}

/// Builds the OnTime rundown for an exam session: room opening, the timed
/// exam itself (with warning/danger timer states before `scheduled_end`),
/// and closing/collection — mirroring the sections of `templates/ata.tex`.
pub fn build_rundown_entries(exam: &ExamApplication) -> Vec<OntimeEntry> {
    let duration_ms = exam.schedule.duration_minutes as u64 * 60_000;
    let time_warning = DEFAULT_WARNING_MS.min(duration_ms / 2);
    let time_danger = DEFAULT_DANGER_MS.min(duration_ms / 4);

    vec![
        milestone(
            "ABERTURA",
            format!("Abertura da sala — {}", exam.room),
            format!("{} — Turma(s) {}", exam.course, exam.classes),
        ),
        OntimeEntry {
            kind: "event",
            title: exam.exam_name.clone(),
            cue: Some("PROVA".to_string()),
            note: Some(format!(
                "{} — Prof. {} — Sala {}",
                exam.application_type, exam.professor, exam.room
            )),
            time_start: Some(ms_from_midnight(exam.schedule.scheduled_start)),
            duration: Some(duration_ms),
            timer_type: Some("count-down"),
            end_action: Some("none"),
            time_warning: Some(time_warning),
            time_danger: Some(time_danger),
        },
        milestone(
            "ENCERRAMENTO",
            "Encerramento e recolhimento das provas".to_string(),
            format!(
                "Conferir presentes, provas recolhidas e folhas extras — Turma(s) {}",
                exam.classes
            ),
        ),
    ]
}

pub fn rundown_entries_json(exam: &ExamApplication) -> serde_json::Result<String> {
    serde_json::to_string_pretty(&build_rundown_entries(exam))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::template::{Schedule, Student};
    use chrono::{NaiveDate, NaiveTime};

    fn sample_application(duration_minutes: u32) -> ExamApplication {
        ExamApplication {
            course: "Introdução à Programação".to_string(),
            semester: "2026/2".to_string(),
            professor: "Ada Lovelace".to_string(),
            classes: "A".to_string(),
            exam_name: "1ª Avaliação".to_string(),
            application_type: "Regular".to_string(),
            date: NaiveDate::from_ymd_opt(2026, 8, 27).unwrap(),
            schedule: Schedule {
                scheduled_start: NaiveTime::from_hms_opt(18, 30, 0).unwrap(),
                scheduled_end: NaiveTime::from_hms_opt(20, 10, 0).unwrap(),
                duration_minutes,
            },
            room: "Sala 100 - Prédio 43".to_string(),
            students: Vec::<Student>::new(),
            exit_rows: 2,
        }
    }

    #[test]
    fn builds_opening_exam_and_closing_entries() {
        let exam = sample_application(100);
        let entries = build_rundown_entries(&exam);

        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].kind, "milestone");
        assert_eq!(entries[1].kind, "event");
        assert_eq!(entries[2].kind, "milestone");
    }

    #[test]
    fn converts_start_time_and_duration_to_milliseconds() {
        let exam = sample_application(100);
        let entries = build_rundown_entries(&exam);
        let exam_event = &entries[1];

        // 18:30:00 = 66600 s after midnight
        assert_eq!(exam_event.time_start, Some(66_600_000));
        assert_eq!(exam_event.duration, Some(100 * 60_000));
        assert_eq!(exam_event.time_warning, Some(10 * 60_000));
        assert_eq!(exam_event.time_danger, Some(2 * 60_000));
    }

    #[test]
    fn clamps_warning_and_danger_for_short_exams() {
        let exam = sample_application(4);
        let entries = build_rundown_entries(&exam);
        let exam_event = &entries[1];

        // 4 min duration: default 10 min warning / 2 min danger both exceed
        // it, so they clamp to half and a quarter of the duration instead.
        assert_eq!(exam_event.time_warning, Some(2 * 60_000));
        assert_eq!(exam_event.time_danger, Some(60_000));
    }
}
