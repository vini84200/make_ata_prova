use askama::Template;
use inquire::{Confirm, Select, Text};
use std::fs::File;
use std::io::Write;
use std::process::Command;

use crate::template::ExamApplication;
use crate::tui::{create_exam_application, edit_exam_application};

mod latex;
mod ontime;
mod template;
mod tui;

enum Action {
    CreateAta,
    OpenAta,
    Exit,
}

impl std::fmt::Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::CreateAta => write!(f, "Criar ata de prova"),
            Action::OpenAta => write!(f, "Abrir ata de prova existente"),
            Action::Exit => write!(f, "Sair"),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let opcao = Select::new(
        "O que deseja fazer?",
        vec![Action::CreateAta, Action::OpenAta, Action::Exit],
    )
    .prompt()?;

    match opcao {
        Action::CreateAta => {
            let application = create_exam_application()?;

            let save_json =
                Confirm::new("Deseja salvar os dados da aplicação de prova em um arquivo JSON?")
                    .with_default(true)
                    .prompt()?;
            if save_json {
                let json_path = inquire::Text::new(
                    "Caminho para salvar o arquivo JSON (ex: application.json):",
                )
                .with_default(&"application.json".to_string())
                .prompt()?;
                let json_data = serde_json::to_string_pretty(&application)?;
                std::fs::write(json_path.clone(), json_data)?;
                println!("Dados da aplicação de prova salvos em {}", json_path);
            }

            post_process(&application)?;
        }
        Action::OpenAta => {
            let json_path = Text::new("Caminho do arquivo JSON da aplicação de prova:")
                .with_default(&"application.json".to_string())
                .prompt()?;
            let json_data = std::fs::read_to_string(json_path.clone())?;
            let mut application: ExamApplication = serde_json::from_str(&json_data)?;
            println!("Dados da aplicação de prova carregados de {}", json_path);

            if Confirm::new("Deseja editar os dados antes de gerar o PDF?")
                .with_default(false)
                .prompt()?
            {
                edit_exam_application(&mut application)?;
            }

            post_process(&application)?;
        }
        Action::Exit => {
            println!("Saindo...");
            return Ok(());
        }
    }

    Ok(())
}

fn post_process(application: &ExamApplication) -> anyhow::Result<()> {
    if Confirm::new("Gerar pdf de ata de prova com os dados informados?")
        .with_default(true)
        .prompt()?
    {
        generate_pdf_from_template(application)?;
    }

    if Confirm::new(
        "Gerar planilha (.xlsx) do cronograma, importável no OnTime (Rundown > Import > Import from spreadsheet)?",
    )
    .with_default(false)
    .prompt()?
    {
        ontime::write_rundown_spreadsheet(application, "cronograma_ontime.xlsx")?;
        println!("Cronograma salvo em cronograma_ontime.xlsx");
    }

    if Confirm::new(
        "Gerar cronograma em JSON, para uso direto na ferramenta MCP ontime_batch_create_entries do OnTime?",
    )
    .with_default(false)
    .prompt()?
    {
        generate_ontime_schedule(application)?;
    }

    Ok(())
}

fn generate_pdf_from_template(application: &ExamApplication) -> Result<(), anyhow::Error> {
    let template = template::AtaTemplate {
        exam: application,
        generated_at: chrono::Local::now().format("%d/%m/%Y %H:%M").to_string(),
    };
    let latex = template.render()?;
    std::fs::write("ata.tex", latex.clone())?;
    println!("LaTeX file generated: ata.tex");
    println!("Starting PDF generation...");
    let pdf_data: Vec<u8> = tectonic::latex_to_pdf(&latex).map_err(|e| {
        let chain: Vec<String> = e.iter().map(|item| item.to_string()).collect();
        anyhow::anyhow!("Falha ao compilar ata.tex para PDF:\n{}", chain.join("\ncausado por: "))
    })?;
    println!("PDF generated successfully, saving to ata.pdf...");
    let mut file = File::create("ata.pdf")?;
    file.write_all(&pdf_data)?;
    println!("PDF saved as ata.pdf");
    println!("Opening PDF...");
    open_pdf("ata.pdf");
    Ok(())
}

fn generate_ontime_schedule(application: &ExamApplication) -> anyhow::Result<()> {
    let json = ontime::rundown_entries_json(application)?;
    std::fs::write("cronograma_ontime.json", &json)?;
    println!("Cronograma salvo em cronograma_ontime.json");
    println!(
        "Este arquivo NÃO é um projeto do OnTime — é só o array `entries` para a ferramenta MCP ontime_batch_create_entries. Para importar direto pela interface do OnTime, use a planilha .xlsx."
    );
    Ok(())
}

fn open_pdf(path: &str) {
    let result = if cfg!(target_os = "macos") {
        Command::new("open").arg(path).spawn()
    } else if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", path]).spawn()
    } else {
        Command::new("xdg-open").arg(path).spawn()
    };

    if let Err(e) = result {
        eprintln!("Não foi possível abrir {path} automaticamente: {e}");
    }
}
