use crate::ANKIKAJI_TABLE;
use crate::models::Card;
use rusqlite::{OptionalExtension, Transaction};
use rusqlite::{Result as SqlResult, Row};
use sea_query::{
    Alias, ColumnDef, Expr, ExprTrait, Iden, OnConflict, Query, SqliteQueryBuilder, Table,
};
use sea_query_rusqlite::RusqliteBinder;

impl Card {
    pub fn from_row(row: &Row) -> SqlResult<Self> {
        Ok(Self {
            index: row.get("Index")?,
            word: row.get("Word")?,
            word_with_reading: row.get("Word With Reading")?,
            definition: row.get("Definition")?,
            example_sentence: row.get("Example Sentence")?,
            sentence_with_reading: row.get("Sentence With Reading")?,
            translation_sentence: row.get("Sentence Translation")?,
            kanji: row.get("Kanji")?,
            word_audio: row.get("Word Audio")?,
            sentence_audio: row.get("Sentence Audio")?,
            image: row.get("Image")?,
            export: row.get("Export")?,
        })
    }
}

#[derive(Iden)]
enum CardTable {
    Table,
    Index,
    Word,
    #[iden = "Word With Reading"]
    WordWithReading,
    #[iden = "Definition"]
    Definition,
    #[iden = "Example Sentence"]
    ExampleSentence,
    #[iden = "Sentence With Reading"]
    SentenceWithReading,
    #[iden = "Sentence Translation"]
    SentenceTranslation,
    #[iden = "Kanji"]
    Kanji,
    #[iden = "Word Audio"]
    WordAudio,
    #[iden = "Sentence Audio"]
    SentenceAudio,
    #[iden = "Image"]
    Image,
    #[iden = "Export"]
    Export,
}

// ## DB Oper ##
pub fn create_table(conn: &rusqlite::Connection) -> rusqlite::Result<()> {
    let sql = Table::create()
        .table(ANKIKAJI_TABLE.as_str())
        .if_not_exists()
        .col(
            ColumnDef::new(CardTable::Index)
                .integer()
                .auto_increment()
                .primary_key(),
        )
        .col(
            ColumnDef::new(CardTable::Word)
                .string()
                .not_null()
                .unique_key(),
        )
        .col(ColumnDef::new(CardTable::WordWithReading).string())
        .col(ColumnDef::new(CardTable::Definition).text())
        .col(ColumnDef::new(CardTable::ExampleSentence).text())
        .col(ColumnDef::new(CardTable::SentenceWithReading).text())
        .col(ColumnDef::new(CardTable::SentenceTranslation).text())
        .col(ColumnDef::new(CardTable::Kanji).boolean().default(0))
        .col(ColumnDef::new(CardTable::WordAudio).string())
        .col(ColumnDef::new(CardTable::SentenceAudio).string())
        .col(ColumnDef::new(CardTable::Image).string())
        .col(ColumnDef::new(CardTable::Export).boolean().default(0))
        .build(SqliteQueryBuilder);

    conn.execute(&sql, [])?;
    Ok(())
}

pub fn insert_card(conn: &rusqlite::Connection, card: Card) -> rusqlite::Result<()> {
    let (sql, values) = Query::insert()
        .into_table(ANKIKAJI_TABLE.as_str())
        .columns([
            CardTable::Word,
            CardTable::WordWithReading,
            CardTable::Definition,
            CardTable::ExampleSentence,
            CardTable::SentenceWithReading,
            CardTable::SentenceTranslation,
            CardTable::Kanji,
            CardTable::WordAudio,
            CardTable::SentenceAudio,
            CardTable::Image,
            CardTable::Export,
        ])
        .values_panic([
            card.word.into(),
            card.word_with_reading.into(),
            card.definition.into(),
            card.example_sentence.into(),
            card.sentence_with_reading.into(),
            card.translation_sentence.into(),
            card.kanji.into(),
            card.word_audio.into(),
            card.sentence_audio.into(),
            card.image.into(),
            card.export.into(),
        ])
        .build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, &*values.as_params())?;
    Ok(())
}

pub fn insert_or_update_card_ingore_null_values_tx(
    conn: &Transaction,
    card: Card,
) -> rusqlite::Result<()> {
    let (sql, values) = Query::insert()
        .into_table(ANKIKAJI_TABLE.as_str())
        .columns([
            CardTable::Word,
            CardTable::WordWithReading,
            CardTable::Definition,
            CardTable::ExampleSentence,
            CardTable::SentenceWithReading,
            CardTable::SentenceTranslation,
            CardTable::Kanji,
            CardTable::WordAudio,
            CardTable::SentenceAudio,
            CardTable::Image,
            CardTable::Export,
        ])
        .values_panic([
            card.word.clone().into(),
            card.word_with_reading.clone().into(),
            card.definition.clone().into(),
            card.example_sentence.clone().into(),
            card.sentence_with_reading.clone().into(),
            card.translation_sentence.clone().into(),
            card.kanji.clone().into(),
            card.word_audio.clone().into(),
            card.sentence_audio.clone().into(),
            card.image.clone().into(),
            card.export.clone().into(),
        ])
        .on_conflict(
            OnConflict::column(CardTable::Word)
                .values([
                    (
                        CardTable::WordWithReading,
                        Expr::col((Alias::new("excluded"), CardTable::WordWithReading))
                            .if_null(Expr::col(CardTable::WordWithReading)),
                    ),
                    (
                        CardTable::Definition,
                        Expr::col((Alias::new("excluded"), CardTable::Definition))
                            .if_null(Expr::col(CardTable::Definition)),
                    ),
                    (
                        CardTable::ExampleSentence,
                        Expr::col((Alias::new("excluded"), CardTable::ExampleSentence))
                            .if_null(Expr::col(CardTable::ExampleSentence)),
                    ),
                    (
                        CardTable::SentenceWithReading,
                        Expr::col((Alias::new("excluded"), CardTable::SentenceWithReading))
                            .if_null(Expr::col(CardTable::SentenceWithReading)),
                    ),
                    (
                        CardTable::SentenceTranslation,
                        Expr::col((Alias::new("excluded"), CardTable::SentenceTranslation))
                            .if_null(Expr::col(CardTable::SentenceTranslation)),
                    ),
                    (
                        CardTable::Kanji,
                        Expr::col((Alias::new("excluded"), CardTable::Kanji))
                            .if_null(Expr::col(CardTable::Kanji)),
                    ),
                    (
                        CardTable::WordAudio,
                        Expr::col((Alias::new("excluded"), CardTable::WordAudio))
                            .if_null(Expr::col(CardTable::WordAudio)),
                    ),
                    (
                        CardTable::SentenceAudio,
                        Expr::col((Alias::new("excluded"), CardTable::SentenceAudio))
                            .if_null(Expr::col(CardTable::SentenceAudio)),
                    ),
                    (
                        CardTable::Image,
                        Expr::col((Alias::new("excluded"), CardTable::Image))
                            .if_null(Expr::col(CardTable::Image)),
                    ),
                    (
                        CardTable::Export,
                        Expr::col((Alias::new("excluded"), CardTable::Export))
                            .if_null(Expr::col(CardTable::Export)),
                    ),
                ])
                .to_owned(),
        )
        .build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, &*values.as_params())?;
    Ok(())
}

pub fn insert_or_update_card_tx(conn: &Transaction, card: Card) -> rusqlite::Result<()> {
    let (sql, values) = Query::insert()
        .into_table(ANKIKAJI_TABLE.as_str())
        .columns([
            CardTable::Word,
            CardTable::WordWithReading,
            CardTable::Definition,
            CardTable::ExampleSentence,
            CardTable::SentenceWithReading,
            CardTable::SentenceTranslation,
            CardTable::Kanji,
            CardTable::WordAudio,
            CardTable::SentenceAudio,
            CardTable::Image,
            CardTable::Export,
        ])
        .values_panic([
            card.word.clone().into(),
            card.word_with_reading.clone().into(),
            card.definition.clone().into(),
            card.example_sentence.clone().into(),
            card.sentence_with_reading.clone().into(),
            card.translation_sentence.clone().into(),
            card.kanji.clone().into(),
            card.word_audio.clone().into(),
            card.sentence_audio.clone().into(),
            card.image.clone().into(),
            card.export.clone().into(),
        ])
        .on_conflict(
            OnConflict::column(CardTable::Word)
                .update_columns([
                    CardTable::WordWithReading,
                    CardTable::Definition,
                    CardTable::ExampleSentence,
                    CardTable::SentenceWithReading,
                    CardTable::SentenceTranslation,
                    CardTable::Kanji,
                    CardTable::WordAudio,
                    CardTable::SentenceAudio,
                    CardTable::Image,
                    CardTable::Export,
                ])
                .to_owned(),
        )
        .build_rusqlite(SqliteQueryBuilder);

    conn.execute(&sql, &*values.as_params())?;
    Ok(())
}

pub fn get_card_by_word(conn: &rusqlite::Connection, word: &str) -> rusqlite::Result<Option<Card>> {
    let (sql, values) = Query::select()
        .columns([
            CardTable::Index,
            CardTable::Word,
            CardTable::WordWithReading,
            CardTable::Definition,
            CardTable::ExampleSentence,
            CardTable::SentenceWithReading,
            CardTable::SentenceTranslation,
            CardTable::Kanji,
            CardTable::WordAudio,
            CardTable::SentenceAudio,
            CardTable::Image,
            CardTable::Export,
        ])
        .from(ANKIKAJI_TABLE.as_str())
        .and_where(Expr::col(CardTable::Word).eq(word))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare(&sql)?;
    let card_opt = stmt
        .query_row(&*values.as_params(), |row| Card::from_row(row))
        .optional()?;

    Ok(card_opt)
}

pub fn get_unexported_cards(conn: &rusqlite::Connection) -> rusqlite::Result<Vec<Card>> {
    let (sql, values) = Query::select()
        .columns([
            CardTable::Index,
            CardTable::Word,
            CardTable::WordWithReading,
            CardTable::Definition,
            CardTable::ExampleSentence,
            CardTable::SentenceWithReading,
            CardTable::SentenceTranslation,
            CardTable::Kanji,
            CardTable::WordAudio,
            CardTable::SentenceAudio,
            CardTable::Image,
            CardTable::Export,
        ])
        .from(ANKIKAJI_TABLE.as_str())
        .and_where(Expr::col(CardTable::Export).eq(0))
        .build_rusqlite(SqliteQueryBuilder);

    let mut stmt = conn.prepare(&sql)?;

    let cards = stmt
        .query_map(&*values.as_params(), |row| Card::from_row(row))?
        .collect::<rusqlite::Result<Vec<Card>>>()?;

    Ok(cards)
}

pub fn mark_cards_exported(
    conn: &mut rusqlite::Connection,
    words: &[String],
) -> rusqlite::Result<()> {
    let tx = conn.transaction()?;
    for word in words {
        let (sql, values) = Query::update()
            .table(ANKIKAJI_TABLE.as_str())
            .value(CardTable::Export, 1)
            .and_where(Expr::col(CardTable::Word).eq(word))
            .build_rusqlite(SqliteQueryBuilder);

        tx.execute(&sql, &*values.as_params())?;
    }
    tx.commit()?;
    Ok(())
}
