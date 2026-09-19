use std::io::{ErrorKind, Read, Seek, SeekFrom, Write};

use crate::error::RotateError;

/// 16KiB buffer size
const BUFF_SIZE: usize = 16 * 1024;

pub type RotateResult<T> = Result<T, RotateError>;

/// wrap LSB -> MSB 0x01 -> 0x80
pub fn rotate_right(mut source: impl Read + Seek, mut destination: impl Write) -> RotateResult<()> {
    let mut last_byte = [0u8];
    source.seek(SeekFrom::End(-1))?;
    source.read_exact(&mut last_byte)?;
    //extract lsb
    let mut carry_bit = extract_lsb_and_rotate_to_msb(last_byte[0]);

    source.rewind()?;

    let mut buf = [0u8; BUFF_SIZE];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for byte in &buf[..n] {
                    let next_carry_bit = extract_lsb_and_rotate_to_msb(*byte);
                    let new_byte = (byte >> 1) | carry_bit;
                    destination.write_all(&[new_byte])?;
                    carry_bit = next_carry_bit;
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }
    destination.flush()?;
    Ok(())
}

// wrap MSB -> LSB 0x08 -> 0x01
pub fn rotate_left(mut source: impl Read, mut destination: impl Write) -> RotateResult<()> {
    let mut first_byte = [0u8];
    source.read_exact(&mut first_byte)?;
    //extract lsb
    let mut prev = first_byte[0];
    let wrap = extract_msb_and_rotate_to_lsb(prev);

    let mut buf = [0u8; BUFF_SIZE];

    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                for byte in &buf[..n] {
                    let new_byte = (prev << 1) | extract_msb_and_rotate_to_lsb(*byte);
                    destination.write_all(&[new_byte])?;
                    prev = *byte;
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => return Err(e.into()),
        }
    }

    destination.write_all(&[prev << 1 | wrap])?;
    destination.flush()?;

    Ok(())
}

fn extract_lsb_and_rotate_to_msb(value: u8) -> u8 {
    (value & 0x01) << 7
}

fn extract_msb_and_rotate_to_lsb(value: u8) -> u8 {
    (value & 0x80) >> 7
}

#[cfg(test)]
mod tests {
    use crate::{rotate_left, rotate_right};
    use std::io::Cursor;

    #[test]
    fn test_rotate_left() {
        let input = Cursor::new([0x81u8, 0x81u8]);
        let mut output = Vec::new();
        rotate_left(input, &mut output).unwrap();
        assert_eq!(output, [0x03, 0x03]);
    }

    #[test]
    fn test_rotate_right() {
        let input = Cursor::new([0x81u8, 0x81u8]);
        let mut output = Vec::new();
        rotate_right(input, &mut output).unwrap();
        assert_eq!(output, [0xC0, 0xC0]);
    }
}
