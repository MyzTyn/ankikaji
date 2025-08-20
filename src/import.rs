use crate::models::FieldSpec;

impl FieldSpec {
    pub fn from_yaml(filename: &str) -> std::io::Result<Vec<FieldSpec>> {
        let file_content = std::fs::read_to_string(filename)?;

        Ok(vec![])
    }
}
