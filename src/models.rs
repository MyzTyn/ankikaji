use std::{collections::HashMap, error::Error};

use sea_query::{Alias, ColumnDef};
use serde::Deserialize;

use crate::annotate;

#[derive(Debug, Clone, Default, Deserialize)]
pub enum FieldType {
    #[default]
    String,
    Text,
    Boolean,
    Integer,
}

// Card Field Spec
#[derive(Deserialize)]
pub struct FieldSpec {
    pub name: String,
    pub field_type: FieldType,
    pub metadata: HashMap<String, String>,
}

impl FieldSpec {
    #[inline]
    pub fn get_alias(&self) -> Alias {
        Alias::new(self.name.as_str())
    }

    #[inline]
    pub fn get_col(&self) -> ColumnDef {
        let mut col = match self.field_type {
            FieldType::Integer => ColumnDef::new(self.get_alias()).integer().to_owned(),
            FieldType::Text => ColumnDef::new(self.get_alias()).text().to_owned(),
            FieldType::Boolean => ColumnDef::new(self.get_alias()).boolean().to_owned(),
            _ => ColumnDef::new(self.get_alias()).string().to_owned(),
        };

        if (self.is_primary_key()) {
            col.primary_key();
        }

        if (self.is_auto_increment()) {
            col.auto_increment();
        }

        if (self.is_not_null()) {
            col.not_null();
        }

        if (self.is_unique()) {
            col.unique_key();
        }

        col
    }

    #[inline]
    pub fn is_primary_key(&self) -> bool {
        self.metadata.contains_key("Primary Key")
    }

    #[inline]
    pub fn is_key(&self) -> bool {
        self.metadata.contains_key("Key")
    }

    #[inline]
    pub fn is_auto_increment(&self) -> bool {
        self.metadata.contains_key("Auto Increment")
    }

    #[inline]
    pub fn is_not_null(&self) -> bool {
        self.metadata.contains_key("Not Null")
    }

    #[inline]
    pub fn is_unique(&self) -> bool {
        self.metadata.contains_key("Unique")
    }

    #[inline]
    pub fn is_image(&self) -> bool {
        self.metadata.contains_key("Image")
    }

    #[inline]
    pub fn autoruby(&self) -> Option<String> {
        self.metadata
            .get("Autoruby")
            .and_then(|s| s.parse::<String>().ok())
    }

    #[inline]
    pub fn true_if_exists(&self) -> Option<String> {
        self.metadata
            .get("True If Exists")
            .and_then(|s| s.parse::<String>().ok())
    }
}

// Card MetaData
#[derive(Deserialize)]
pub struct CardMetadata {
    pub name: String,
    pub fields: Vec<FieldSpec>,
}

impl CardMetadata {
    // Helper Fn
    pub fn get_main_key(&self) -> &FieldSpec {
        self.fields
            .iter()
            .filter(|field| field.is_key())
            .next()
            .unwrap()
    }

    pub fn get_data_from_record(
        &self,
        record: &HashMap<String, String>,
    ) -> Option<(Vec<Alias>, Vec<String>, Alias)> {
        // Early return statement
        if self.fields.is_empty() {
            return None;
        }

        let mut aliases = Vec::new();
        let mut values = Vec::new();
        let mut key: Option<Alias> = None;
        for field in &self.fields {
            // The data that have be inputted
            if field.is_not_null() && !field.is_auto_increment() {
                match record.get(&field.name) {
                    Some(val) => {
                        if field.is_key() {
                            key = Some(field.get_alias())
                        }
                        aliases.push(field.get_alias());
                        values.push(val.clone());
                    }
                    _ => return None, // missing or empty → invalid
                }
            } else {
                if let Some(val) = record.get(&field.name) {
                    if field.is_key() {
                        key = Some(field.get_alias())
                    }
                    aliases.push(field.get_alias());
                    values.push(val.clone());
                }
            }
        }

        // case: all fields nullable → require at least one alias
        if aliases.is_empty() && key.is_some() {
            return None;
        }

        Some((aliases, values, key.unwrap()))
    }
    // Preprocess the get_data_from_record
    pub fn preprocess_data(
        &self,
        fields: &Vec<Alias>,
        values: &Vec<String>,
    ) -> Option<(Vec<Alias>, Vec<String>)> {
        // Early return statement
        if self.fields.is_empty() {
            return None;
        }

        let mut aliases = fields.clone();
        let mut values = values.clone();

        for field in &self.fields {
            if aliases.contains(&field.get_alias()) && !field.is_image() {
                continue;
            } else if !aliases.contains(&field.get_alias()) && field.is_image() {
                continue;
            }

            let pos = aliases
                .iter()
                .position(|value| value.0 == field.get_alias().0);

            // The input is not filled or missing
            if field.is_image() {
                values[pos.unwrap()] = format!("<img src=\"{}\">", values[pos.unwrap()]);
            } else if let Some(target) = field.autoruby() {
                let target_pos = aliases.iter().position(|value| value.0 == target);
                match target_pos {
                    None => continue,
                    Some(target_pos) => {
                        let result = annotate(values[target_pos].as_str());
                        if result == values[target_pos] {
                            continue;
                        }

                        aliases.push(field.get_alias());
                        values.push(result);
                    }
                }
            } else if let Some(target) = field.true_if_exists() {
                let target_pos = aliases.iter().position(|value| value.0 == target);
                match target_pos {
                    None => continue,
                    Some(target_pos) => {
                        if (!values[target_pos].is_empty()) {
                            aliases.push(field.get_alias());
                            values.push("1".to_string());
                        }
                    }
                }
            }
        }

        // case: all fields nullable → require at least one alias
        if aliases.is_empty() || values.is_empty() {
            return None;
        }

        Some((aliases, values))
    }

    pub fn from_yaml(filename: &str) -> Result<CardMetadata, Box<dyn Error>> {
        let file_content = std::fs::read_to_string(filename)?;
        let card_metadata: CardMetadata = serde_yaml::from_str(&file_content)?;

        Ok(card_metadata)
    }
}
