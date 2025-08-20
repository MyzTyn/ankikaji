use serde::{Deserialize, Serialize};

use crate::annotate;

/*
 * Card Template
 */

#[derive(Debug, Clone, Default)]
pub enum FieldType {
    #[default]
    String,
    Index,
    Boolean,
    Image,
    Integer,
    Autoruby,
}

pub struct FieldSpec {
    pub name: String,
    pub id: String,
    pub optional: bool,
    pub field_type: FieldType,
}

#[derive(Debug, Deserialize)]
pub struct YamlCard {
    pub word: String,
    pub word_with_reading: Option<String>,
    pub definition: Option<String>,
    pub example_sentence: Option<String>,
    pub sentence_with_reading: Option<String>,
    pub translation_sentence: Option<String>,
    pub kanji: Option<bool>,
    pub image: Option<String>,
}

#[derive(Clone, Default, Debug, Serialize)]
pub struct Card {
    pub index: i64,
    pub word: String,
    pub word_with_reading: Option<String>,
    pub definition: Option<String>,
    pub example_sentence: Option<String>,
    pub sentence_with_reading: Option<String>,
    pub translation_sentence: Option<String>,
    pub kanji: Option<bool>,
    pub word_audio: Option<String>,
    pub sentence_audio: Option<String>,
    pub image: Option<String>,
    pub export: bool,
}

impl From<YamlCard> for Card {
    fn from(value: YamlCard) -> Self {
        let word_with_reading = value.word_with_reading.or_else(|| {
            let ann = annotate(&value.word);
            if ann != value.word { Some(ann) } else { None }
        });

        let sentence_with_reading = value.sentence_with_reading.or_else(|| {
            value.example_sentence.as_ref().and_then(|ex| {
                let ann = annotate(ex);
                if ann != *ex { Some(ann) } else { None }
            })
        });
        let image = if let Some(image) = value.image {
            Some(format!("<img src='{image}'>"))
        } else {
            None
        };

        Card {
            kanji: value.kanji.or(Some(word_with_reading.is_some())),
            word: value.word,
            word_with_reading,
            definition: value.definition,
            example_sentence: value.example_sentence,
            sentence_with_reading,
            translation_sentence: value.translation_sentence,
            image,
            ..Default::default()
        }
    }
}
