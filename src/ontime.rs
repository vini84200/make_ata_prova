use chrono::Timelike;
use rust_xlsxwriter::{Format, Workbook};
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

fn hms(total_ms: u64) -> String {
    let total_seconds = total_ms / 1000;
    format!(
        "{:02}:{:02}:{:02}",
        total_seconds / 3600,
        (total_seconds % 3600) / 60,
        total_seconds % 60
    )
}

const SPREADSHEET_HEADERS: [&str; 10] = [
    "Cue",
    "Title",
    "Note",
    "Start",
    "End",
    "Duration",
    "Timer type",
    "End action",
    "Time warning",
    "Time danger",
];

/// Writes the same rundown as `build_rundown_entries` to an .xlsx file using
/// the column headers Ontime's spreadsheet importer recognizes (Rundown >
/// Import > Import from spreadsheet). Unlike `rundown_entries_json`, this
/// file is directly importable through the Ontime UI — it is a plain
/// worksheet, not an Ontime project file.
pub fn write_rundown_spreadsheet(exam: &ExamApplication, path: &str) -> anyhow::Result<()> {
    let entries = build_rundown_entries(exam);

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet().set_name("Rundown")?;
    let bold = Format::new().set_bold();

    for (col, header) in SPREADSHEET_HEADERS.iter().enumerate() {
        worksheet.write_with_format(0, col as u16, *header, &bold)?;
    }

    for (i, entry) in entries.iter().enumerate() {
        let row = (i + 1) as u32;

        worksheet.write(row, 0, entry.cue.as_deref().unwrap_or_default())?;
        worksheet.write(row, 1, entry.title.as_str())?;
        worksheet.write(row, 2, entry.note.as_deref().unwrap_or_default())?;

        if entry.kind == "milestone" {
            worksheet.write(row, 6, "milestone")?;
            continue;
        }

        let start = entry.time_start.unwrap_or(0);
        let duration = entry.duration.unwrap_or(0);
        worksheet.write(row, 3, hms(start))?;
        worksheet.write(row, 4, hms(start + duration))?;
        worksheet.write(row, 5, hms(duration))?;
        worksheet.write(row, 6, entry.timer_type.unwrap_or("count-down"))?;
        worksheet.write(row, 7, entry.end_action.unwrap_or("none"))?;
        if let Some(warning) = entry.time_warning {
            worksheet.write(row, 8, hms(warning))?;
        }
        if let Some(danger) = entry.time_danger {
            worksheet.write(row, 9, hms(danger))?;
        }
    }

    worksheet.autofit();
    workbook.save(path)?;

    Ok(())
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
            features: Default::default(),
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
    fn writes_a_valid_xlsx_file() {
        let exam = sample_application(100);
        let path = std::env::temp_dir().join("make_ata_prova_test_rundown.xlsx");
        let path_str = path.to_str().unwrap();

        write_rundown_spreadsheet(&exam, path_str).unwrap();

        let bytes = std::fs::read(&path).unwrap();
        // .xlsx files are zip archives, which start with the "PK" signature.
        assert_eq!(&bytes[0..2], b"PK");

        std::fs::remove_file(&path).unwrap();
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
