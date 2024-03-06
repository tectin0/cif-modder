pub fn remove_uncertainty_digits(value: &str) -> String {
    let mut value = value.to_string();
    if value.contains('(') {
        let index = value
            .find('(')
            .expect("Failed to find '('. Should not happen.");

        value.truncate(index);
    }
    value
}

pub trait RemoveUncertaintyDigits {
    fn remove_uncertainty_digits(&self) -> String;
}

impl RemoveUncertaintyDigits for str {
    fn remove_uncertainty_digits(&self) -> String {
        remove_uncertainty_digits(self)
    }
}

pub fn precision_of_value(value: &str) -> usize {
    let value_precision = match value.split('.').last() {
        Some(precision) => precision.len(),
        None => 0,
    };

    value_precision
}

pub fn whitespace_between_two_values(line: &str) -> Option<usize> {
    let mut words = line.split_whitespace();
    let key = words.next()?;
    let value = words.next()?;
    let whitespace = line.find(value)? - key.len();
    Some(whitespace)
}

pub fn directory_content_from_path(path: &str) -> anyhow::Result<Vec<String>> {
    let paths = match std::fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                let paths: Vec<Result<String, anyhow::Error>> = std::fs::read_dir(&path)?
                    .map(|entry| {
                        entry.map(|e| {
                            e.path()
                                .to_str()
                                .ok_or_else(|| anyhow::anyhow!("Invalid path"))
                                .map(str::to_string)
                        })
                    })
                    .collect::<Result<Vec<Result<String, anyhow::Error>>, std::io::Error>>()?;

                let paths = paths
                    .into_iter()
                    .collect::<Result<Vec<String>, anyhow::Error>>()?;

                Ok(paths)
            } else {
                Ok(vec![path.to_string()])
            }
        }
        Err(e) => return Err(anyhow::anyhow!("{:#}", e)),
    };

    paths
}

#[cfg(test)]
pub fn find_keyword_in_cif_file(path: &str, keyword: &str) -> anyhow::Result<String> {
    use std::io::BufRead;

    let file = std::fs::File::open(path)?;

    let reader = std::io::BufReader::new(file);
    let lines = reader.lines();

    for line in lines {
        let line = line?;
        let mut words = line.split_whitespace();
        if let Some(key) = words.next() {
            if key == keyword {
                if let Some(value) = words.next() {
                    return Ok(value.to_string());
                }
            }
        }
    }

    Err(anyhow::anyhow!("Keyword not found"))
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_remove_uncertainty_digits() {
        let value = "4.0094(2)";
        let result = super::remove_uncertainty_digits(value);
        assert_eq!(result, "4.0094");
    }

    #[test]
    fn test_find_keyword_in_cif_file() {
        let path = "tests/BaTiO3.cif";
        let keyword = "_cell_length_a";
        let result = super::find_keyword_in_cif_file(path, keyword);
        assert_eq!(result.unwrap(), "4.0094(2)");
    }

    #[test]
    fn test_precision_of_value() {
        let value = "4.0094";
        let result = super::precision_of_value(value);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_whitespace_between_two_values() {
        let line = "_cell_length_a     4.0094(2)";
        let result = super::whitespace_between_two_values(line);
        assert_eq!(result, Some(5));
    }
}
