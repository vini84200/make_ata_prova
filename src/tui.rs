use std::collections::HashSet;
use std::fmt;
use std::ops::Add;

use anyhow::Result;
use chrono::{Datelike, Local, NaiveTime};
use inquire::{Confirm, CustomType, DateSelect, MultiSelect, Select, Text};
use serde::Deserialize;

use crate::template::{AtaFeatures, ExamApplication, Schedule, Student};

/// One toggleable entry in the "funcionalidades extras" selector. Each
/// variant maps to a field (or `Option` field) on `AtaFeatures` — plain
/// booleans for options with no extra data, and a follow-up `Text` prompt for
/// options that need a value (course code, department, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FeatureOption {
    ClassColumn,
    DeliveryTimeColumn,
    SecondProctor,
    CourseCode,
    Department,
    AllowedMaterials,
    FormalClosing,
}

const ALL_FEATURE_OPTIONS: [FeatureOption; 7] = [
    FeatureOption::ClassColumn,
    FeatureOption::DeliveryTimeColumn,
    FeatureOption::SecondProctor,
    FeatureOption::CourseCode,
    FeatureOption::Department,
    FeatureOption::AllowedMaterials,
    FeatureOption::FormalClosing,
];

impl fmt::Display for FeatureOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            FeatureOption::ClassColumn => "Mostrar a turma de cada aluno na lista de presença",
            FeatureOption::DeliveryTimeColumn => {
                "Coluna para hora de entrega individual da prova"
            }
            FeatureOption::SecondProctor => "Assinatura de um segundo fiscal (fiscal auxiliar)",
            FeatureOption::CourseCode => "Código da disciplina",
            FeatureOption::Department => "Instituto/Departamento",
            FeatureOption::AllowedMaterials => "Materiais permitidos / regras da avaliação",
            FeatureOption::FormalClosing => "Local e data por extenso antes da assinatura",
        };
        write!(f, "{label}")
    }
}

fn feature_is_enabled(current: &AtaFeatures, option: FeatureOption) -> bool {
    match option {
        FeatureOption::ClassColumn => current.show_class_column,
        FeatureOption::DeliveryTimeColumn => current.show_delivery_time_column,
        FeatureOption::SecondProctor => current.second_proctor,
        FeatureOption::CourseCode => current.course_code.is_some(),
        FeatureOption::Department => current.department.is_some(),
        FeatureOption::AllowedMaterials => current.allowed_materials.is_some(),
        FeatureOption::FormalClosing => current.closing_city.is_some(),
    }
}

/// Prompts a multi-select of optional ata features, pre-checking whichever
/// ones `current` already has enabled, then collects a follow-up value for
/// every selected option that needs one.
fn prompt_features(current: &AtaFeatures) -> Result<AtaFeatures> {
    let defaults: Vec<usize> = ALL_FEATURE_OPTIONS
        .iter()
        .enumerate()
        .filter(|(_, option)| feature_is_enabled(current, **option))
        .map(|(i, _)| i)
        .collect();

    let selected = MultiSelect::new(
        "Funcionalidades extras da ata (espaço para marcar, enter para confirmar):",
        ALL_FEATURE_OPTIONS.to_vec(),
    )
    .with_default(&defaults)
    .prompt()?;

    let mut features = AtaFeatures {
        show_class_column: selected.contains(&FeatureOption::ClassColumn),
        show_delivery_time_column: selected.contains(&FeatureOption::DeliveryTimeColumn),
        second_proctor: selected.contains(&FeatureOption::SecondProctor),
        ..AtaFeatures::default()
    };

    if selected.contains(&FeatureOption::CourseCode) {
        features.course_code = Some(
            Text::new("Código da disciplina:")
                .with_default(current.course_code.as_deref().unwrap_or(""))
                .prompt()?,
        );
    }

    if selected.contains(&FeatureOption::Department) {
        features.department = Some(
            Text::new("Instituto/Departamento:")
                .with_default(current.department.as_deref().unwrap_or(""))
                .prompt()?,
        );
    }

    if selected.contains(&FeatureOption::AllowedMaterials) {
        features.allowed_materials = Some(
            Text::new("Materiais permitidos / regras da avaliação:")
                .with_default(current.allowed_materials.as_deref().unwrap_or(""))
                .prompt()?,
        );
    }

    if selected.contains(&FeatureOption::FormalClosing) {
        features.closing_city = Some(
            Text::new("Cidade (para \"Cidade, DD de mês de AAAA\" antes da assinatura):")
                .with_default(current.closing_city.as_deref().unwrap_or(""))
                .prompt()?,
        );
    }

    Ok(features)
}

#[derive(Debug, Deserialize)]
struct CsvStudent {
    #[serde(rename = "ID")]
    id: u32,

    #[serde(rename = "nome")]
    name: String,

    #[serde(rename = "turma")]
    class: String,

    email: String,
}

/// Wraps a student together with its index in the list it came from, so a
/// `Select` prompt can hand back the index directly instead of forcing a
/// reverse lookup by formatted label.
struct StudentOption {
    idx: usize,
    label: String,
}

impl fmt::Display for StudentOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.label)
    }
}

fn student_options(students: &[Student]) -> Vec<StudentOption> {
    students
        .iter()
        .enumerate()
        .map(|(idx, s)| StudentOption {
            idx,
            label: format!("{} - {} ({})", s.number, s.name, s.id),
        })
        .collect()
}

fn load_students(path: &str) -> Result<Vec<Student>> {
    let mut rdr = csv::Reader::from_path(path)?;

    let mut students = Vec::new();

    for (i, row) in rdr.deserialize::<CsvStudent>().enumerate() {
        let csv = row?;

        students.push(Student {
            id: csv.id,
            number: i + 1,
            name: csv.name,
            class: csv.class,
            email: csv.email,
            observations: None,
        });
    }

    Ok(students)
}

/// Imports a CSV, preserving `observations` already recorded for students
/// that are re-imported (matched by `id`), instead of discarding them.
fn import_students(path: &str, existing: &[Student]) -> Result<Vec<Student>> {
    let mut imported = load_students(path)?;

    for student in imported.iter_mut() {
        if let Some(previous) = existing.iter().find(|s| s.id == student.id) {
            student.observations = previous.observations.clone();
        }
    }

    warn_duplicate_ids(&imported);

    Ok(imported)
}

fn warn_duplicate_ids(students: &[Student]) {
    let mut seen = HashSet::new();
    let mut duplicated = HashSet::new();

    for s in students {
        if !seen.insert(s.id) {
            duplicated.insert(s.id);
        }
    }

    if !duplicated.is_empty() {
        println!("Atenção: ID(s) duplicado(s) na lista de alunos: {duplicated:?}");
    }
}

pub fn create_exam_application() -> Result<ExamApplication> {
    let course = Text::new("Atividade de Ensino:").prompt()?;

    // Se antes de agosto, ANO/1, senão ANO/2
    let semestre_estimated = if Local::now().month() < 8 {
        format!("{}/1", Local::now().year())
    } else {
        format!("{}/2", Local::now().year())
    };
    let semester = Text::new("Período Letivo:")
        .with_default(&semestre_estimated)
        .prompt()?;

    let professor = Text::new("Professor:").prompt()?;

    let classes = Text::new("Turma(s):").prompt()?;

    let exam_name = Text::new("Avaliação:").prompt()?;

    let application_type = Select::new(
        "Tipo de aplicação:",
        vec!["Regular", "Substituitiva", "Especial"],
    )
    .prompt()?
    .to_string();

    let date = DateSelect::new("Data da avaliação:").prompt()?;

    let room = Text::new("Sala:").prompt()?;
    let predio = Text::new("Prédio:").prompt()?;
    let room = format!("{} - Prédio {}", room.trim(), predio.trim());

    let scheduled_start =
        CustomType::<NaiveTime>::new("Horário de início previsto(HH:MM)").prompt()?;

    let duration_minutes = CustomType::<u32>::new("Duração (minutos)").prompt()?;

    let scheduled_end = CustomType::<NaiveTime>::new("Horário de término previsto(HH:MM)")
        .with_default(scheduled_start.add(chrono::Duration::minutes(duration_minutes as i64)))
        .prompt()?;

    let exit_rows =
        CustomType::<usize>::new("Linhas para o registro de saídas temporárias")
            .with_default(4)
            .prompt()?;

    let features = prompt_features(&AtaFeatures::default())?;

    let students = student_menu(Vec::new())?;

    Ok(ExamApplication {
        course,
        semester,
        professor,
        classes,
        exam_name,
        application_type,
        date,
        room,

        schedule: Schedule {
            scheduled_start,
            scheduled_end,
            duration_minutes,
        },

        students,
        exit_rows,
        features,
    })
}

/// Lets the user revisit an already-built `ExamApplication` (loaded from
/// JSON) before regenerating its PDF: edit the student roster and the number
/// of temporary-exit rows.
pub fn edit_exam_application(application: &mut ExamApplication) -> Result<()> {
    loop {
        let action = Select::new(
            "Editar dados da aplicação",
            vec![
                "Editar alunos",
                "Editar linhas de saídas temporárias",
                "Editar funcionalidades extras",
                "Concluir edição",
            ],
        )
        .prompt()?;

        match action {
            "Editar alunos" => {
                application.students = student_menu(std::mem::take(&mut application.students))?;
            }

            "Editar linhas de saídas temporárias" => {
                application.exit_rows = CustomType::<usize>::new(
                    "Linhas para o registro de saídas temporárias",
                )
                .with_default(application.exit_rows)
                .prompt()?;
            }

            "Editar funcionalidades extras" => {
                application.features = prompt_features(&application.features)?;
            }

            "Concluir edição" => break,

            _ => {}
        }
    }

    Ok(())
}

fn student_menu(mut students: Vec<Student>) -> anyhow::Result<Vec<Student>> {
    loop {
        let action = Select::new(
            "Students",
            vec![
                "Import CSV",
                "Add student",
                "Remove student",
                "Show students",
                "Edit student",
                "Finish",
            ],
        )
        .prompt()?;

        match action {
            "Import CSV" => {
                let path = Text::new("CSV file:").prompt()?;

                if students.is_empty()
                    || Confirm::new(
                        "Já existem alunos na lista. Importar vai mesclar por ID, mantendo observações já registradas. Continuar?",
                    )
                    .with_default(true)
                    .prompt()?
                {
                    students = import_students(&path, &students)?;
                    renumber(&mut students);
                }
            }

            "Add student" => {
                let student = prompt_student()?;

                if students.iter().any(|s| s.id == student.id) {
                    println!("Atenção: já existe um aluno com ID {}.", student.id);
                }

                students.push(student);

                renumber(&mut students);
            }

            "Remove student" => {
                remove_student(&mut students)?;
            }

            "Show students" => {
                println!();

                for s in &students {
                    println!("{:3} {:8} {}", s.number, s.id, s.name);
                }

                println!();
            }

            "Edit student" => {
                edit_student(&mut students)?;
            }

            "Finish" => break,

            _ => {}
        }
    }

    Ok(students)
}

fn prompt_student() -> anyhow::Result<Student> {
    Ok(Student {
        id: CustomType::new("ID").prompt()?,

        number: 0,

        name: Text::new("Name").prompt()?,

        class: Text::new("Class").prompt()?,

        email: Text::new("Email").prompt()?,
        observations: {
            let has_observations = Confirm::new("Any observations?").prompt()?;

            if has_observations {
                Some(Text::new("Observations:").prompt()?)
            } else {
                None
            }
        },
    })
}

fn remove_student(students: &mut Vec<Student>) -> anyhow::Result<()> {
    if students.is_empty() {
        return Ok(());
    }

    let selected = Select::new("Remove student", student_options(students)).prompt()?;

    students.remove(selected.idx);

    renumber(students);

    Ok(())
}

fn edit_student(students: &mut [Student]) -> anyhow::Result<()> {
    if students.is_empty() {
        return Ok(());
    }

    let selected = Select::new("Edit student", student_options(students)).prompt()?;

    add_observations(&mut students[selected.idx])?;

    Ok(())
}

fn add_observations(student: &mut Student) -> anyhow::Result<()> {
    let obs = Text::new("Observations:").prompt()?;

    student.observations = Some(obs);

    Ok(())
}

fn renumber(students: &mut [Student]) {
    for (i, s) in students.iter_mut().enumerate() {
        s.number = i + 1;
    }
}
