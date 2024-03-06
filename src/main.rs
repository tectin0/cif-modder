use anyhow::Context;
use cif_modder::directory_content_from_path;
use clap::Parser;

fn main() {
    let args = cif_modder::Args::parse();

    let is_verbose = args.verbose;

    simple_logger::SimpleLogger::new()
        .with_level(if is_verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init()
        .unwrap();

    if args.examples {
        print!(indoc::indoc!(
            "
            Examples:

            cif-modder --cif path/to/cif --instructions \"_cell_length_a + 1; _cell_length_a * 2; _cell_length_b -- 5.00; 70 -- _cell_angle_alpha -- 120\"

            - `path/to/cif` is the path to the CIF file or directory containing CIF files.
            - `_cell_length_a + 1` adds 1 to the value of _cell_length_a.
            - `_cell_length_a * 2` multiplies the value of _cell_length_a by 2. This is applied after the previous instruction.
            - `_cell_length_b -- 5.00` sets the value to a random number between 5.00 and the original value of _cell_length_b.
            - `70 -- _cell_angle_alpha -- 120` sets the value to a random number between 70 and 120.

            cif-modder -c path/to/cif -i \"a + 1; a * 2; b -- 5.00; 70 -- alpha -- 120\"

            - `a` and `b` are the same as `_cell_length_a` and `_cell_length_b`.
            - `alpha` is the same as `_cell_angle_alpha`.
            - `-c` is the short form of `--cif`. `-i` is the short form of `--instructions`.

            cif_modder -c path/to/cif -i \"path/to/instructions.txt\"

            - The instructions can also be read from a file.
            - Valid delimiters are `;`, `,`, and `\\n`.

            List of currently recognized CIF keywords:
            `_cell_length_a`, `_cell_length_b`, `_cell_length_c`, `_cell_angle_alpha`, `_cell_angle_beta`, `_cell_angle_gamma`, `_cell_volume`

            Short keywords:
            `a`, `b`, `c`, `alpha`, `beta`, `gamma`, `volume`

            List of all possible operators:

            `+` adds a value to the current value.
            `-` subtracts a value from the current value.
            `*` multiplies the current value by a value.
            `/` divides the current value by a value.
            `^` raises the current value to the power of a value.
            `--` sets the current value to a random number between the current value and a value or between two values.
            "
        ));

        std::process::exit(0);
    }

    log::debug!("{:#?}", args);

    let instructions = match args.instructions {
        Some(instructions) => instructions,
        None => {
            log::error!("Error: No instructions provided.");
            std::process::exit(1);
        }
    };

    let path = match args.cif {
        Some(path) => path,
        None => {
            log::error!("Error: No path provided.");
            std::process::exit(1);
        }
    };

    let is_instructions_a_file = std::fs::metadata(&instructions).is_ok();

    log::debug!("is_instructions_file: {}", is_instructions_a_file);

    let instructions = match is_instructions_a_file {
        true => {
            let instructions = match cif_modder::Instructions::from_file(&instructions) {
                Ok(instructions) => instructions,
                Err(e) => {
                    log::error!("{:#}", e);
                    std::process::exit(1);
                }
            };
            instructions
        }
        false => {
            let instructions = cif_modder::Instructions::from_string(&instructions);
            instructions
        }
    };

    log::debug!("Instructions: {:#?}", instructions);

    let paths: Vec<String> = match std::fs::metadata(&path) {
        Ok(metadata) => {
            if metadata.is_dir() {
                match directory_content_from_path(&path) {
                    Ok(paths) => paths,
                    Err(e) => {
                        log::error!("{:#}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                vec![path.to_string()]
            }
        }
        Err(_) => {
            log::error!("Error: The path does not exist.");
            std::process::exit(1);
        }
    }
    .into_iter()
    .filter(|path| path.ends_with(".cif") && !path.contains("_modified.cif"))
    .collect();

    log::debug!("Paths: {:#?}", paths);

    for path in paths {
        let new_lines =
            match cif_modder::apply_instructions_to_cif_file(&path, instructions.clone())
                .context("Could not apply instructions to CIF file.")
            {
                Ok(new_lines) => new_lines,
                Err(e) => {
                    log::error!("{:#}", e);
                    std::process::exit(1);
                }
            };

        let new_path = path.replace(".cif", "_modified.cif");

        match std::fs::write(&new_path, new_lines.join("\n"))
            .context("Could not write modified CIF file.")
        {
            Ok(_) => (),
            Err(e) => {
                log::error!("{:#}", e);
                std::process::exit(1);
            }
        }

        log::debug!("Wrote to {}", new_path);
    }
}
