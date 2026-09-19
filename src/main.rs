use std::{
    fs::{self, File, OpenOptions},
    io::{BufReader, BufWriter, ErrorKind},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    error::RotateError,
    rotate::{RotateResult, rotate_left, rotate_right},
};

mod error;
mod rotate;

fn main() -> RotateResult<()> {
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            print_help();
            return Err(e);
        }
    };

    let input = open_input(&args.input_path).map_err(RotateError::IO)?;
    let OutputBufferAndTempFile {
        buffer: output,
        temmp_file,
    } = open_temporary_output(&args.output_path).map_err(RotateError::IO)?;
    match args.direction {
        Direction::Left => rotate_left(input, output)?,
        Direction::Right => rotate_right(input, output)?,
    };
    // Safety: rename in linux is atomic
    std::fs::rename(temmp_file, args.output_path).map_err(RotateError::IO)
}

fn open_input(path: &PathBuf) -> std::io::Result<BufReader<File>> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Input is not a file",
        ));
    }

    if metadata.len() == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "Input was empty",
        ));
    }

    Ok(BufReader::new(File::open(path)?))
}

fn open_temporary_output(path: &Path) -> std::io::Result<OutputBufferAndTempFile> {
    if path.exists() && !path.is_file() {
        return Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            "output path exists and is not a regular file",
        ));
    }

    let file_name = path.file_name().ok_or_else(|| {
        std::io::Error::new(ErrorKind::InvalidInput, "output path has no file name")
    })?;

    let mut temp_file_name = file_name.to_os_string();
    temp_file_name.push(".tmp");

    match OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file_name)
    {
        Err(err) if err.kind() == ErrorKind::PermissionDenied => Err(std::io::Error::new(
            std::io::ErrorKind::PermissionDenied,
            "Could not open output for writing",
        )),
        Err(e) => Err(e),
        Ok(file) => Ok(OutputBufferAndTempFile {
            buffer: BufWriter::new(file),
            temmp_file: PathBuf::from(temp_file_name),
        }),
    }
}

fn parse_args() -> RotateResult<RotateArgs> {
    let direction_arg = std::env::args()
        .nth(1)
        .ok_or(RotateError::ParseArgumentsError(
            "Could not parse direction arg",
        ))?;

    if matches!(direction_arg.as_str(), "-h" | "--help") {
        print_help();
        std::process::exit(0);
    }

    let input_path_arg = std::env::args()
        .nth(2)
        .ok_or(RotateError::ParseArgumentsError(
            "Could not parse input path",
        ))?;
    let output_path_arg = std::env::args()
        .nth(3)
        .ok_or(RotateError::ParseArgumentsError(
            "Could not parser output path",
        ))?;

    let input_path = Path::new(&input_path_arg);
    let output_path = Path::new(&output_path_arg);

    Ok(RotateArgs {
        direction: direction_arg.parse().map_err(|_| {
            std::io::Error::new(
                ErrorKind::InvalidInput,
                format!("Could not parsed direction: {direction_arg}"),
            )
        })?,
        input_path: input_path.to_path_buf(),
        output_path: output_path.to_path_buf(),
    })
}

fn print_help() {
    eprintln!(
        "\
Usage: rotate <left|right> <input-file> <output-file>
"
    );
}

#[derive(Debug, Default)]
struct RotateArgs {
    direction: Direction,
    input_path: PathBuf,
    output_path: PathBuf,
}

#[derive(Debug, Default, Clone, Copy)]
enum Direction {
    #[default]
    Left,
    Right,
}

impl FromStr for Direction {
    type Err = RotateError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "left" => Ok(Self::Left),
            "right" => Ok(Self::Right),
            _ => Err(RotateError::ParseArgumentsError(
                "invalid direction argument",
            )),
        }
    }
}

pub struct OutputBufferAndTempFile {
    pub buffer: BufWriter<File>,
    pub temmp_file: PathBuf,
}
