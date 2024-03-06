# cif-modder
Modify parameters in cif files

# Usage

```sh
cif-modder --cif path/to/cif --instructions "_cell_length_a + 1; _cell_length_a * 2; _cell_length_b -- 5.00; 70 -- _cell_angle_alpha -- 120"
```

- `path/to/cif` is the path to the CIF file or directory containing CIF files.
- `_cell_length_a + 1` adds 1 to the value of _cell_length_a.
- `_cell_length_a * 2` multiplies the value of _cell_length_a by 2. This is applied after the previous instruction.
- `_cell_length_b -- 5.00` sets the value to a random number between 5.00 and the original value of _cell_length_b.
- `70 -- _cell_angle_alpha -- 120` sets the value to a random number between 70 and 120.

```sh
cif-modder -c path/to/cif -i "a + 1; a * 2; b -- 5.00; 70 -- alpha -- 120"
```

- `a` and `b` are the same as `_cell_length_a` and `_cell_length_b`.
- `alpha` is the same as `_cell_angle_alpha`.
- `-c` is the short form of `--cif`. `-i` is the short form of `--instructions`.

```sh
cif_modder -c path/to/cif -i "path/to/instructions.txt"
```

- The instructions can also be read from a file.
- Valid delimiters are `;`, `,`, and `\\n`.

List of currently recognized CIF keywords:
`_cell_length_a`, `_cell_length_b`, `_cell_length_c`, `_cell_angle_alpha`, `_cell_angle_beta`, `_cell_angle_gamma`, `_cell_volume`

Short keywords:
`a`, `b`, `c`, `alpha`, `beta`, `gamma`, `volume`

List of all possible operators:

- `+` adds a value to the current value.
- `-` subtracts a value from the current value.
- `*` multiplies the current value by a value.
- `/` divides the current value by a value.
- `^` raises the current value to the power of a value.
- `--` sets the current value to a random number between the current value and a value or between two values.
