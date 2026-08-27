use askama::Template;
use chrono::{Datelike, NaiveDate, NaiveTime};
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

/// Optional additions to the generated ata, toggled through the "funcionalidades
/// extras" selector in the TUI. Everything here is either off/`None` by default,
/// so old JSON files without a `features` key still deserialize fine via
/// `#[serde(default)]` on `ExamApplication::features`.
#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct AtaFeatures {
    pub show_class_column: bool,
    pub show_delivery_time_column: bool,
    pub second_proctor: bool,
    pub course_code: Option<String>,
    pub department: Option<String>,
    pub allowed_materials: Option<String>,
    pub closing_city: Option<String>,
}

impl AtaFeatures {
    /// Column count added to the presence table beyond the fixed
    /// Nº/Matrícula/Nome/Presença/Assinatura columns.
    fn extra_presence_columns(&self) -> usize {
        self.show_class_column as usize + self.show_delivery_time_column as usize
    }

    /// `longtable` column spec for the presence table, expanded with the
    /// Turma/Entrega columns when enabled. Nome/Assinatura widths are
    /// hand-tuned per combination so the row always fits within the page's
    /// text width; the dotted signature fill itself is sized with
    /// `\linewidth` in the template, so it always matches whatever width is
    /// picked here.
    pub fn presence_table_colspec(&self) -> String {
        let (name_width, signature_width) =
            match (self.show_class_column, self.show_delivery_time_column) {
                (false, false) => (5.2, 5.8),
                (true, false) => (4.1, 5.4),
                (false, true) => (4.9, 4.3),
                (true, true) => (3.8, 3.9),
            };

        let mut spec = String::from(r"c>{\centering\arraybackslash}p{1.5cm}");
        spec.push_str(&format!("p{{{name_width}cm}}"));
        if self.show_class_column {
            spec.push_str(r">{\centering\arraybackslash}p{1.1cm}");
        }
        spec.push('c');
        spec.push_str(&format!("p{{{signature_width}cm}}"));
        if self.show_delivery_time_column {
            spec.push_str(r">{\centering\arraybackslash}p{1.4cm}");
        }
        spec
    }

    /// How many table columns the "Observações" note under a student row
    /// should span: Matrícula, Nome, Presença, Assinatura, plus any enabled
    /// extra columns — i.e. every column except Nº.
    pub fn observation_span(&self) -> usize {
        4 + self.extra_presence_columns()
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

    #[serde(default)]
    pub features: AtaFeatures,
}

const MONTHS_PT_BR: [&str; 12] = [
    "janeiro",
    "fevereiro",
    "março",
    "abril",
    "maio",
    "junho",
    "julho",
    "agosto",
    "setembro",
    "outubro",
    "novembro",
    "dezembro",
];

impl ExamApplication {
    pub fn date_br(&self) -> String {
        self.date.format("%d/%m/%Y").to_string()
    }

    /// Formal, written-out date (e.g. "27 de agosto de 2026") for the
    /// optional "local e data" closing line before the signature block.
    pub fn date_extenso(&self) -> String {
        format!(
            "{} de {} de {}",
            self.date.day(),
            MONTHS_PT_BR[(self.date.month() - 1) as usize],
            self.date.year()
        )
    }
}

#[derive(Template)]
#[template(path = "ata.tex", escape = "tex")]
pub struct AtaTemplate<'a> {
    pub exam: &'a ExamApplication,
    pub generated_at: String,
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
            features: AtaFeatures::default(),
        }
    }

    fn render(exam: &ExamApplication) -> String {
        AtaTemplate {
            exam,
            generated_at: "27/08/2026 10:00".to_string(),
        }
        .render()
        .unwrap()
    }

    /// JSON files saved before the `features` field existed don't have that
    /// key at all; `#[serde(default)]` must fill it in as all-off instead of
    /// failing to deserialize "Open existing ata".
    #[test]
    fn deserializes_json_saved_before_features_existed() {
        let json = r#"{
            "course": "Introdução à Programação",
            "semester": "2026/2",
            "professor": "Ada Lovelace",
            "classes": "A",
            "exam_name": "1ª Avaliação",
            "application_type": "Regular",
            "date": "2026-08-27",
            "schedule": {
                "scheduled_start": "18:30:00",
                "scheduled_end": "20:10:00",
                "duration_minutes": 100
            },
            "room": "Sala 100 - Prédio 43",
            "students": [],
            "exit_rows": 2
        }"#;

        let exam: ExamApplication = serde_json::from_str(json).unwrap();
        assert!(!exam.features.show_class_column);
        assert!(exam.features.course_code.is_none());
    }

    #[test]
    fn date_and_time_render_in_brazilian_format() {
        let exam = sample_application();
        assert_eq!(exam.date_br(), "27/08/2026");
        assert_eq!(exam.date_extenso(), "27 de agosto de 2026");
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
        let latex = render(&exam);
        let pdf = tectonic::latex_to_pdf(&latex).expect("ata.tex should compile to PDF");
        assert!(pdf.starts_with(b"%PDF"));
    }

    /// Same as `renders_to_a_valid_pdf`, but with every optional feature
    /// turned on, to make sure the conditional column widths and extra
    /// sections still add up to a compilable document.
    #[test]
    #[ignore]
    fn renders_to_a_valid_pdf_with_all_features_enabled() {
        let mut exam = sample_application();
        exam.students = (1..=15)
            .map(|n| Student {
                id: 200000 + n as u32,
                number: n,
                name: format!("Aluno de Teste Número {n} da Silva"),
                class: if n % 2 == 0 { "B" } else { "A" }.to_string(),
                email: format!("aluno{n}@example.com"),
                observations: if n == 5 {
                    Some("Chegou atrasado".to_string())
                } else {
                    None
                },
            })
            .collect();
        exam.features = AtaFeatures {
            show_class_column: true,
            show_delivery_time_column: true,
            second_proctor: true,
            course_code: Some("INF01202".to_string()),
            department: Some("Instituto de Informática".to_string()),
            allowed_materials: Some("Prova sem consulta, calculadora simples permitida".to_string()),
            closing_city: Some("Porto Alegre".to_string()),
        };
        let latex = render(&exam);
        let pdf = tectonic::latex_to_pdf(&latex).expect("ata.tex should compile to PDF");
        assert!(pdf.starts_with(b"%PDF"));
    }

    #[test]
    fn renders_and_escapes_student_data() {
        let exam = sample_application();
        let rendered = render(&exam);

        assert!(rendered.contains("27/08/2026"));
        assert!(rendered.contains(r"Maria \& Souza\_1"));
        assert!(rendered.contains(r"50\% de presença"));
        assert!(rendered.matches(r"\section*{Horários}").count() == 1);
    }

    #[test]
    fn optional_features_are_off_by_default_in_the_rendered_document() {
        let exam = sample_application();
        let rendered = render(&exam);

        assert!(!rendered.contains(r"\textbf{Turma}"));
        assert!(!rendered.contains("Código"));
        assert!(!rendered.contains("Fiscal Auxiliar"));
    }
}
