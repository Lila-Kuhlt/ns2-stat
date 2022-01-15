pub mod types;

fn load_data(path: &str) -> std::io::Result<Vec<types::GameStats>> {
    std::fs::read_dir(path)?
        .flat_map(|res| res.map(|e| e.path()))
        .filter_map(|path| path.is_file().then(|| std::fs::read_to_string(&path)))
        .flatten()
        .map(|path| {
            types::GameStats::from_json(&path)
                .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("{e}")))
        })
        .collect::<std::io::Result<Vec<_>>>()
}

#[cfg(test)]
mod tests {
    use crate::types::GameStats;

    #[test]
    fn test_data_parsable() -> std::io::Result<()> {
        super::load_data("test_data").map(|_| ())
    }
}
