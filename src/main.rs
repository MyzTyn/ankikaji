use autoruby::annotate::Annotator;
use clap::Parser;
use std::sync::{LazyLock, OnceLock};

use crate::{
    db::{
        create_table, get_unexported_cards, insert_card,
        insert_or_update_card_ingore_null_values_tx, mark_cards_exported,
    },
    models::{Card, YamlCard},
};

mod db;
mod import;
mod models;

pub static AUTORUBY: OnceLock<Annotator> = OnceLock::new();

pub static ANKIKAJI_DB: LazyLock<String> = LazyLock::new(|| std::env::var("ANKIKAJI_DB").unwrap());
pub static ANKIKAJI_TABLE: LazyLock<String> =
    LazyLock::new(|| std::env::var("ANKIKAJI_TABLE").unwrap());

pub struct SimpleFormat;
impl autoruby::format::Format for SimpleFormat {
    fn format(&self, base: &str, text: &str) -> String {
        format!(" {base}[{text}] ")
    }
}
fn annotate(text: &str) -> String {
    AUTORUBY
        .get_or_init(|| Annotator::new_with_integrated_dictionary())
        .annotate(text)
        .render(&autoruby::select::heuristic::All, &SimpleFormat)
        .trim()
        .to_string()
}

#[derive(Parser, Debug)]
#[command(version, about = "AnkiKaji CLI — Simple Japanese card manager")]
enum Command {
    Add {
        word: String,
        definition: String,
        example_sentence: String,
        translation_sentence: Option<String>,
    },
    ImportYaml,
    ExportCsv {
        #[arg(short, long, default_value = "export.csv")]
        file: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse();

    let mut conn = rusqlite::Connection::open(ANKIKAJI_DB.as_str())?;

    create_table(&conn)?;

    match command {
        Command::Add {
            word,
            definition,
            example_sentence,
            translation_sentence,
        } => {
            // Your existing add logic here (similar to before)
            let word_with_reading = annotate(&word);
            let sentence_with_reading = annotate(&example_sentence);
            let kanji = word_with_reading != word;

            let card = Card {
                word,
                word_with_reading: kanji.then(|| word_with_reading),
                definition: Some(definition),
                example_sentence: Some(example_sentence),
                sentence_with_reading: Some(sentence_with_reading),
                translation_sentence,
                kanji: Some(kanji),
                ..Default::default()
            };

            insert_card(&conn, card)?;
            println!("✅ Card saved to database.");
        }
        Command::ImportYaml => {
            import_yaml(&mut conn, "cards.yaml")?;
        }
        Command::ExportCsv { file } => {
            let cards = get_unexported_cards(&conn)?;
            if cards.is_empty() {
                println!("No new cards to export.");
                return Ok(());
            }
            export_to_csv(&cards, &file)?;
            let words: Vec<String> = cards.iter().map(|c| c.word.clone()).collect();
            mark_cards_exported(&mut conn, &words)?;
            println!("✅ Exported {} cards to '{}'", words.len(), file);
        }
    }

    Ok(())
}

fn import_yaml(
    conn: &mut rusqlite::Connection,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(filename)?;
    let cards: Vec<YamlCard> = serde_yaml::from_str(&file_content)?;

    let tx = conn.transaction()?; // start transaction for batch

    for record in cards {
        let card: Card = record.into();

        println!("Upserted card: {}", card.word);

        insert_or_update_card_ingore_null_values_tx(&tx, card)?;
    }

    tx.commit()?; // commit transaction

    Ok(())
}

fn export_to_csv(cards: &[Card], filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut wtr = csv::Writer::from_path(filename)?;
    for card in cards {
        let kanji = match card.kanji {
            None => "",
            Some(value) => {
                if value {
                    "1"
                } else {
                    "0"
                }
            }
        };
        wtr.write_record(&[
            &card.index.to_string(),
            &card.word,
            card.word_with_reading.as_deref().unwrap_or(""),
            card.definition.as_deref().unwrap_or(""),
            card.example_sentence.as_deref().unwrap_or(""),
            card.sentence_with_reading.as_deref().unwrap_or(""),
            card.translation_sentence.as_deref().unwrap_or(""),
            &kanji,
            card.word_audio.as_deref().unwrap_or(""),
            card.sentence_audio.as_deref().unwrap_or(""),
            card.image.as_deref().unwrap_or(""),
        ])?;
    }
    wtr.flush()?;
    Ok(())
}
