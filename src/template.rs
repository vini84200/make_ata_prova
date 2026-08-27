use askama::Template;
use chrono::{NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Student {
    pub id: u32,
    pub number: usize,
    pub name: String,
    pub class: String,
    pub email: String,
    pub observations: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Schedule {
    pub scheduled_start: NaiveTime,
    pub scheduled_end: NaiveTime,
    pub duration_minutes: u32,
}

impl Schedule {
    pub fn scheduled_start_br(&self) -> String {
        self.scheduled_start.format("%H:%M").to_string()
    }

    pub fn scheduled_end_br(&self) -> String {
        self.scheduled_end.format("%H:%M").to_string()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExamApplication {
    pub course: String,
    pub semester: String,
    pub professor: String,
    pub classes: String,

    pub exam_name: String,
    pub application_type: String,

    pub date: NaiveDate,

    pub schedule: Schedule,

    pub room: String,

    pub students: Vec<Student>,

    pub exit_rows: usize,
}

impl ExamApplication {
    pub fn date_br(&self) -> String {
        self.date.format("%d/%m/%Y").to_string()
    }
}

#[derive(Template)]
#[template(path = "ata.tex", escape = "tex")]
pub struct AtaTemplate<'a> {
    pub exam: &'a ExamApplication,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_application() -> ExamApplication {
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
                duration_minutes: 100,
            },
            room: "Sala 100 - Prédio 43".to_string(),
            students: vec![Student {
                id: 1,
                number: 1,
                name: "Maria & Souza_1".to_string(),
                class: "A".to_string(),
                email: "maria@example.com".to_string(),
                observations: Some("50% de presença".to_string()),
            }],
            exit_rows: 2,
        }
    }

    #[test]
    fn date_and_time_render_in_brazilian_format() {
        let exam = sample_application();
        assert_eq!(exam.date_br(), "27/08/2026");
        assert_eq!(exam.schedule.scheduled_start_br(), "18:30");
        assert_eq!(exam.schedule.scheduled_end_br(), "20:10");
    }

    /// Compiles the rendered template with the real tectonic engine. Marked
    /// `#[ignore]` because it downloads/uses the tectonic bundle and is slow;
    /// run explicitly with `cargo test -- --ignored` after touching
    /// `templates/ata.tex`.
    #[test]
    #[ignore]
    fn renders_to_a_valid_pdf() {
        let exam = sample_application();
        let latex = AtaTemplate { exam: &exam }.render().unwrap();
        let pdf = tectonic::latex_to_pdf(&latex).expect("ata.tex should compile to PDF");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn renders_and_escapes_student_data() {
        let exam = sample_application();
        let rendered = AtaTemplate { exam: &exam }.render().unwrap();

        assert!(rendered.contains("27/08/2026"));
        assert!(rendered.contains(r"Maria \& Souza\_1"));
        assert!(rendered.contains(r"50\% de presença"));
        assert!(rendered.matches(r"\section*{Horários}").count() == 1);
    }
}
