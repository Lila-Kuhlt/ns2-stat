pub mod types;

#[cfg(test)]
mod tests {
    use crate::types::GameStats;

    #[test]
    fn test_data_parsable() -> std::io::Result<()> {
        use std::fs;

        for entry in fs::read_dir("test_data")? {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            if let Err(e) = GameStats::from_json(&fs::read_to_string(&path)?) {
                panic!("{path:?} is not a parsable file\nError: {e}");
            }
        }

        Ok(())
    }
}
