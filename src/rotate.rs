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
                for byte in &mut buf[..n] {
                    let next_carry_bit = extract_lsb_and_rotate_to_msb(*byte);
                    *byte = (*byte >> 1) | carry_bit;
                    carry_bit = next_carry_bit;
                }
                destination.write_all(&buf[..n])?;
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
                for byte in &mut buf[..n] {
                    let original_byte = *byte;
                    *byte = (prev << 1) | extract_msb_and_rotate_to_lsb(original_byte);
                    prev = original_byte;
                }
                destination.write_all(&buf[..n])?;
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
    fn rotate_left_one_byte() {
        let mut output = Vec::new();
        rotate_left(Cursor::new([0x81u8]), &mut output).unwrap();
        assert_eq!(output, [0x03]);
    }
    #[test]
    fn rotate_left_two_bytes() {
        // 1000_0001 0100_0000  ->  0000_0010 1000_0001
        let mut output = Vec::new();
        rotate_left(Cursor::new([0x81u8, 0x40]), &mut output).unwrap();
        assert_eq!(output, [0x02, 0x81]);
    }
    #[test]
    fn rotate_right_one_byte() {
        let mut output = Vec::new();
        rotate_right(Cursor::new([0x81u8]), &mut output).unwrap();
        assert_eq!(output, [0xC0]);
    }
    #[test]
    fn rotate_right_two_bytes() {
        // 1000_0001 0100_0000  ->  0100_0000 1010_0000
        let mut output = Vec::new();
        rotate_right(Cursor::new([0x81u8, 0x40]), &mut output).unwrap();
        assert_eq!(output, [0x40, 0xA0]);
    }
}
