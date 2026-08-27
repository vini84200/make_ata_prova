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
