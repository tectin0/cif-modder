mod arguments;
mod instructions;
mod utilities;

use std::{
    cell::RefCell,
    fs::File,
    io::{BufRead, BufReader},
};

use rand::{rngs::StdRng, SeedableRng};

pub use arguments::Args;

pub use instructions::Instruction;
pub use instructions::Instructions;

pub use instructions::Operator;

pub use utilities::directory_content_from_path;

use utilities::whitespace_between_two_values;

thread_local! {
    #[cfg(not(test))]
    pub static RNG: RefCell<StdRng> = RefCell::new(StdRng::from_entropy());
    #[cfg(test)]
    pub static RNG: RefCell<StdRng> = RefCell::new(StdRng::seed_from_u64(0));
}

pub const KEYWORDS: [&str; 7] = [
    "_cell_length_a",
    "_cell_length_b",
    "_cell_length_c",
    "_cell_angle_alpha",
    "_cell_angle_beta",
    "_cell_angle_gamma",
    "_cell_volume",
];

pub const SHORT_KEYWORDS: [&str; 7] = ["a", "b", "c", "alpha", "beta", "gamma", "volume"];

pub fn apply_instructions_to_cif_file(
    path: &str,
    instructions: Instructions,
) -> anyhow::Result<Vec<String>> {
    let mut new_lines = Vec::new();

    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut lines = reader.lines();

    let mut modified_lines_counter = 0;

    while let Some(Ok(line)) = lines.next() {
        let mut modified_line = None;

        let mut words = line.split_whitespace();

        if let Some(key) = words.next() {
            if KEYWORDS.contains(&key) {
                if let Some(value) = words.next() {
                    let whitespace_count = match whitespace_between_two_values(&line) {
                        Some(whitespace) => whitespace,
                        None => {
                            log::error!(
                                "Failed to find whitespace between key and value in {}. Skipping line.",
                                line
                            );
                            continue;
                        }
                    };
                    let whitespace = " ".repeat(whitespace_count);

                    let new_value = instructions.apply(key, value.to_string())?;

                    match new_value {
                        Some(new_value) => {
                            let line_result = format!("{}{}{}", key, whitespace, new_value);

                            log::debug!("{} -> {}", line, line_result);

                            modified_line = Some(line_result);

                            modified_lines_counter += 1;
                        }
                        None => (),
                    }
                }
            }
        }

        if let Some(modified_line) = modified_line {
            new_lines.push(modified_line);
        } else {
            new_lines.push(line);
        }
    }

    log::debug!("Modified {} lines in {}", modified_lines_counter, path);

    Ok(new_lines)
}

#[cfg(test)]
mod tests {
    use crate::instructions::Instructions;

    #[test]
    fn test_cif_map_from_cif_file() {
        let instructions: Instructions =
            "a + 1.0\nb * 2.0\nc - 1.0\nalpha + 1.0\n45.0 -- beta -- 90.0\ngamma / 2.0".into();

        let new_lines = super::apply_instructions_to_cif_file("tests/BaTiO3.cif", instructions)
            .expect("Failed to modify CIF file");

        assert_eq!(new_lines[27], "_cell_length_a                     5.0094");
        assert_eq!(new_lines[28], "_cell_length_b                     8.0188");
        assert_eq!(new_lines[29], "_cell_length_c                     3.0094");
        assert_eq!(new_lines[30], "_cell_angle_alpha                  91.00");

        assert_eq!(new_lines[31], "_cell_angle_beta                   77.90");

        assert_eq!(new_lines[32], "_cell_angle_gamma                  45.00");
    }
}
