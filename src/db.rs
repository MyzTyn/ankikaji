use std::collections::HashMap;

use crate::{
    ANKIKAJI_TABLE,
    models::{CardMetadata, FieldSpec},
};
use sea_query::{Alias, ColumnDef, Expr, ExprTrait, OnConflict, Query, SqliteQueryBuilder, Table};
use sea_query_rusqlite::{RusqliteBinder, RusqliteValues};

// pub fn insert_or_update_card_ingore_null_values_tx(
//     conn: &Transaction,
//     card: Card,
// ) -> rusqlite::Result<()> {
//     let (sql, values) = Query::insert()
//         .into_table(ANKIKAJI_TABLE.as_str())
//         .columns([
//             CardTable::Word,
//             CardTable::WordWithReading,
//             CardTable::Definition,
//             CardTable::ExampleSentence,
//             CardTable::SentenceWithReading,
//             CardTable::SentenceTranslation,
//             CardTable::Kanji,
//             CardTable::WordAudio,
//             CardTable::SentenceAudio,
//             CardTable::Image,
//             CardTable::Export,
//         ])
//         .values_panic([
//             card.word.clone().into(),
//             card.word_with_reading.clone().into(),
//             card.definition.clone().into(),
//             card.example_sentence.clone().into(),
//             card.sentence_with_reading.clone().into(),
//             card.translation_sentence.clone().into(),
//             card.kanji.clone().into(),
//             card.word_audio.clone().into(),
//             card.sentence_audio.clone().into(),
//             card.image.clone().into(),
//             card.export.clone().into(),
//         ])
//         .on_conflict(
//             OnConflict::column(CardTable::Word)
//                 .values([
//                     (
//                         CardTable::WordWithReading,
//                         Expr::col((Alias::new("excluded"), CardTable::WordWithReading))
//                             .if_null(Expr::col(CardTable::WordWithReading)),
//                     ),
//                     (
//                         CardTable::Definition,
//                         Expr::col((Alias::new("excluded"), CardTable::Definition))
//                             .if_null(Expr::col(CardTable::Definition)),
//                     ),
//                     (
//                         CardTable::ExampleSentence,
//                         Expr::col((Alias::new("excluded"), CardTable::ExampleSentence))
//                             .if_null(Expr::col(CardTable::ExampleSentence)),
//                     ),
//                     (
//                         CardTable::SentenceWithReading,
//                         Expr::col((Alias::new("excluded"), CardTable::SentenceWithReading))
//                             .if_null(Expr::col(CardTable::SentenceWithReading)),
//                     ),
//                     (
//                         CardTable::SentenceTranslation,
//                         Expr::col((Alias::new("excluded"), CardTable::SentenceTranslation))
//                             .if_null(Expr::col(CardTable::SentenceTranslation)),
//                     ),
//                     (
//                         CardTable::Kanji,
//                         Expr::col((Alias::new("excluded"), CardTable::Kanji))
//                             .if_null(Expr::col(CardTable::Kanji)),
//                     ),
//                     (
//                         CardTable::WordAudio,
//                         Expr::col((Alias::new("excluded"), CardTable::WordAudio))
//                             .if_null(Expr::col(CardTable::WordAudio)),
//                     ),
//                     (
//                         CardTable::SentenceAudio,
//                         Expr::col((Alias::new("excluded"), CardTable::SentenceAudio))
//                             .if_null(Expr::col(CardTable::SentenceAudio)),
//                     ),
//                     (
//                         CardTable::Image,
//                         Expr::col((Alias::new("excluded"), CardTable::Image))
//                             .if_null(Expr::col(CardTable::Image)),
//                     ),
//                     (
//                         CardTable::Export,
//                         Expr::col((Alias::new("excluded"), CardTable::Export))
//                             .if_null(Expr::col(CardTable::Export)),
//                     ),
//                 ])
//                 .to_owned(),
//         )
//         .build_rusqlite(SqliteQueryBuilder);

//     conn.execute(&sql, &*values.as_params())?;
//     Ok(())
// }

// pub fn get_card_by_word(conn: &rusqlite::Connection, word: &str) -> rusqlite::Result<Option<Card>> {
//     let (sql, values) = Query::select()
//         .columns([
//             CardTable::Index,
//             CardTable::Word,
//             CardTable::WordWithReading,
//             CardTable::Definition,
//             CardTable::ExampleSentence,
//             CardTable::SentenceWithReading,
//             CardTable::SentenceTranslation,
//             CardTable::Kanji,
//             CardTable::WordAudio,
//             CardTable::SentenceAudio,
//             CardTable::Image,
//             CardTable::Export,
//         ])
//         .from(ANKIKAJI_TABLE.as_str())
//         .and_where(Expr::col(CardTable::Word).eq(word))
//         .build_rusqlite(SqliteQueryBuilder);

//     let mut stmt = conn.prepare(&sql)?;
//     let card_opt = stmt
//         .query_row(&*values.as_params(), |row| Card::from_row(row))
//         .optional()?;

//     Ok(card_opt)
// }

// Helper FN

impl CardMetadata {
    pub fn export_csv(
        &self,
        cards: &[HashMap<String, String>],
        filename: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut wtr = csv::Writer::from_path(filename)?;
        for card in cards {
            for key in self.fields.iter().as_ref() {
                if let Some(value) = card.get(&key.name) {
                    wtr.write_field(value)?;
                } else {
                    wtr.write_field("")?;
                }
            }
            wtr.write_record(None::<&[u8]>)?;
        }
        wtr.flush()?;
        Ok(())
    }
}

// DB OP
// ToDo: Use rusqlite value instead of string

impl CardMetadata {
    pub fn create_table_sql(&self) -> String {
        let mut temp = Table::create()
            .table(ANKIKAJI_TABLE.as_str())
            .if_not_exists()
            .to_owned();

        self.fields.iter().for_each(|field| {
            temp.col(field.get_col());
        });
        // Export Col
        temp.col(ColumnDef::new(Alias::new("Export")).boolean().default(0));

        temp.build(SqliteQueryBuilder)
    }

    pub fn insert_card_sql(&self, values: &Vec<String>) -> (String, RusqliteValues) {
        let temp = Query::insert()
            .into_table(ANKIKAJI_TABLE.as_str())
            .columns(get_all_aliases(&self.fields))
            .values_panic(get_all_values(values))
            .to_owned();

        temp.build_rusqlite(SqliteQueryBuilder)
    }

    pub fn insert_or_update_card_sql(
        &self,
        key: &Alias,
        values: &Vec<String>,
    ) -> (String, RusqliteValues) {
        let temp = Query::insert()
            .into_table(ANKIKAJI_TABLE.as_str())
            .columns(get_all_aliases(&self.fields).chain(std::iter::once(Alias::new("Export"))))
            .values_panic(get_all_values(values).chain(std::iter::once(0.into())))
            .on_conflict(
                OnConflict::column(key.clone())
                    .update_columns(get_all_aliases(&self.fields).filter(|alias| alias.0 != key.0))
                    .update_column(Alias::new("Export"))
                    .to_owned(),
            )
            .to_owned();

        temp.build_rusqlite(SqliteQueryBuilder)
    }

    pub fn insert_or_update_card_with_fields_sql(
        &self,
        key: &Alias,
        fields: &Vec<Alias>,
        values: &Vec<String>,
    ) -> (String, RusqliteValues) {
        let mut cols = fields.clone();
        cols.push(Alias::new("Export"));

        let temp = Query::insert()
            .into_table(ANKIKAJI_TABLE.as_str())
            .columns(cols.clone())
            .values_panic(get_all_values(values).chain(std::iter::once(0.into())))
            .on_conflict(
                OnConflict::column(key.clone())
                    .update_columns(cols.iter().filter(|alias| alias.0 != key.0).cloned())
                    .to_owned(),
            )
            .to_owned();

        temp.build_rusqlite(SqliteQueryBuilder)
    }

    pub fn get_unexported_cards_sql(&self) -> (String, RusqliteValues) {
        let temp = Query::select()
            .columns(get_all_aliases(&self.fields))
            .column(Alias::new("Export"))
            .from(ANKIKAJI_TABLE.as_str())
            .and_where(Expr::col(Alias::new("Export")).eq(false))
            .to_owned();

        temp.build_rusqlite(SqliteQueryBuilder)
    }

    pub fn mark_unexported_cards_tx(
        &self,
        key: &Alias,
        words: &[String],
    ) -> Vec<(String, RusqliteValues)> {
        let mut tx = Vec::<(String, RusqliteValues)>::new();
        for word in words {
            let temp = Query::update()
                .table(ANKIKAJI_TABLE.as_str())
                .value(Alias::new("Export"), true)
                .and_where(Expr::col(key.clone()).eq(word))
                .to_owned();

            tx.push(temp.build_rusqlite(SqliteQueryBuilder));
        }

        tx
    }
}

#[inline]
fn get_all_values(values: &Vec<String>) -> impl Iterator<Item = Expr> + '_ {
    values.iter().map(|value| value.into())
}

#[inline]
fn get_all_aliases(fields: &[FieldSpec]) -> impl Iterator<Item = Alias> + '_ {
    fields.iter().map(|field| field.get_alias())
}
