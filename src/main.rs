use autoruby::annotate::Annotator;
use clap::Parser;
use rusqlite::{Row, types::ValueRef};
use std::{
    collections::HashMap,
    sync::{LazyLock, OnceLock},
};

use crate::models::CardMetadata;

mod db;
mod models;

// JP Annotator
pub static AUTORUBY: OnceLock<Annotator> = OnceLock::new();

// SQL file name
pub static ANKIKAJI_DB: LazyLock<String> = LazyLock::new(|| {
    std::env::var("ANKIKAJI_DB").expect("Environment variable ANKIKAJI_DB not set")
});

// SQL table
pub static ANKIKAJI_TABLE: LazyLock<String> = LazyLock::new(|| {
    std::env::var("ANKIKAJI_DB").expect("Environment variable ANKIKAJI_DB not set")
});

// JP Annotator format
pub struct SimpleFormat;
impl autoruby::format::Format for SimpleFormat {
    fn format(&self, base: &str, text: &str) -> String {
        format!(" {base}[{text}] ")
    }
}

// JP Annotator fn
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
    ImportYaml,
    ExportCsv {
        #[arg(short, long, default_value = "export.csv")]
        file: String,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = Command::parse();

    let mut conn = rusqlite::Connection::open(ANKIKAJI_DB.as_str())?;
    let card_metadata = CardMetadata::from_yaml("jp-template.yaml")?;

    // Create Table if need to
    {
        let sql = card_metadata.create_table_sql();
        conn.execute(&sql, [])?;
    }

    match command {
        Command::ImportYaml => {
            import_yaml(&mut conn, card_metadata, "cards.yaml")?;
        }
        Command::ExportCsv { file } => {
            let (sql, values) = card_metadata.get_unexported_cards_sql();
            let mut stmt = conn.prepare(&sql)?;
            let cards = stmt
                .query_map(&*values.as_params(), |row| row_to_hashmap(row))?
                .collect::<rusqlite::Result<Vec<HashMap<String, String>>>>()?;
            drop(stmt);

            if cards.is_empty() {
                println!("No new cards to export.");
                return Ok(());
            }

            let key = card_metadata.get_main_key();

            card_metadata.export_csv(&cards, &file)?;

            let words: Vec<String> = cards
                .iter()
                .map(|c| c.get(key.name.as_str()).unwrap().clone())
                .collect();

            // Mark all cards as exported
            let sqls_values = card_metadata.mark_unexported_cards_tx(&key.get_alias(), &words);
            let tx = conn.transaction()?;
            for (sql, values) in sqls_values {
                tx.execute(&sql, &*values.as_params())?;
            }
            tx.commit()?;

            println!("✅ Exported {} cards to '{}'", words.len(), file);
        }
    }

    Ok(())
}

// Import records from yaml to SQLite DB
fn import_yaml(
    conn: &mut rusqlite::Connection,
    card_metadata: CardMetadata,
    filename: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_content = std::fs::read_to_string(filename)?;
    let cards: Vec<HashMap<String, String>> = serde_yaml::from_str(&file_content)?;

    let tx = conn.transaction()?; // start transaction for batch

    for record in cards.iter().as_ref() {
        let card = card_metadata.get_data_from_record(&record);

        if card.is_none() {
            eprintln!("⚠️ Skipped invalid record: {:?}", record);
            continue;
        }

        let (fields, values, key) = card.unwrap();
        let result = card_metadata.preprocess_data(&fields, &values);
        if result.is_none() {
            continue;
        }
        let (fields, values) = result.unwrap();

        let (sql, params) =
            card_metadata.insert_or_update_card_with_fields_sql(&key, &fields, &values);

        tx.execute(sql.as_str(), &*params.as_params())?;
    }

    tx.commit()?;

    println!("Total Upserted: {}", cards.len());

    Ok(())
}

// Convert Row (from SQLite) to Hashmap
fn row_to_hashmap(row: &Row) -> rusqlite::Result<HashMap<String, String>> {
    let mut map = HashMap::new();
    for (i, col_name) in row.as_ref().column_names().iter().enumerate() {
        let val_ref = row.get_ref(i)?;
        let value_str = match val_ref {
            ValueRef::Null => "".to_string(),
            ValueRef::Integer(i) => i.to_string(),
            ValueRef::Real(f) => f.to_string(),
            ValueRef::Text(t) => String::from_utf8_lossy(t)
                .replace("\n", "<br>")
                .trim_end_matches("<br>")
                .to_string(),
            ValueRef::Blob(b) => format!("{:?}", b),
        };
        map.insert(col_name.to_string(), value_str);
    }
    Ok(map)
}
