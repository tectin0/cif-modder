use cif_modder::Instructions;

fn main() {
    simple_logger::SimpleLogger::new()
        .with_level(log::LevelFilter::Debug)
        .init()
        .unwrap();

    const PATH: &str = "tests/BaTiO3.cif";
    const INSTRUCTIONS: &str =
        "a + 1.0\nb * 2.0\nc - 1.0\nalpha + 1.0\n45.0 -- beta -- 90.0\ngamma / 2.0";

    let instructions: Instructions = INSTRUCTIONS.into();

    let new_lines = cif_modder::apply_instructions_to_cif_file(PATH, instructions).unwrap();

    for line in new_lines.into_iter().skip(27).take(7) {
        println!("{}", line);
    }
}
