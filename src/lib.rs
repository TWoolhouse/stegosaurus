use std::{error::Error, fmt, mem::size_of};

/// Ensure the step size is valid
/// Step must be [1, 8] and a factor of 8
macro_rules! debug_assert_step_size {
    ( $step:ident ) => {
        debug_assert!(0 < $step && $step < u8::BITS as usize);
        debug_assert_eq!(u8::BITS as usize % $step, 0);
    };
}

#[derive(Debug)]
pub struct BufferSizeError {
    missing: usize,
}

impl fmt::Display for BufferSizeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "buffer is too small by {} bytes", self.missing)
    }
}

impl Error for BufferSizeError {}

/// Encodes `data` into the `buffer` using the `step` least significant bits
/// # Examples
/// ```rust
/// # use stegosaurus::{BufferSizeError, encode_raw};
/// # fn main() -> Result<(), BufferSizeError> {
/// let msg = "Hello World";
/// let mut buffer = vec![0; 44];
/// encode_raw(&mut buffer, &msg.as_bytes(), 2)?;
/// # Ok(())
/// # }
/// ```
pub fn encode_raw<'a>(
    buffer: &'a mut [u8],
    data: &[u8],
    step: usize,
) -> Result<&'a mut [u8], BufferSizeError> {
    debug_assert_step_size!(step);
    if buffer.len() < (data.len() * step) {
        return Err(BufferSizeError {
            missing: (data.len() * step) - buffer.len(),
        });
    }

    let space = u8::BITS as usize / step;
    let mut it = buffer.iter_mut();
    for byte in data {
        let mut bit_read: u8 = 0;
        for slot in (&mut it).take(space) {
            for bit_write in 0..step {
                let insert = (byte & (1 << bit_read)) >> bit_read;
                let old = (*slot) & (!(1 << bit_write));
                *slot = old | (insert << bit_write);
                bit_read += 1;
            }
        }
    }
    Ok(&mut buffer[data.len() * space..])
}

fn decode_byte(buffer: &[u8], step: usize) -> Option<u8> {
    debug_assert_step_size!(step);
    let mut current: u8 = 0;
    let mut bit = 0;
    for slot in buffer {
        for bit_read in 0..step {
            current <<= 1;
            current |= (*slot & (1 << bit_read)) >> bit_read;
            bit += 1;
        }
    }
    if bit == 8 {
        Some(current.reverse_bits())
    } else {
        None
    }
}

/// Decodes the data from the `buffer` from the `step` least significant bits
/// # Examples
/// ```rust
/// # use std::{error::Error, str};
/// # use stegosaurus::{encode_raw, decode_raw};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = "Hello World";
/// let mut buffer = vec![0; 44];
/// encode_raw(&mut buffer, &input.as_bytes(), 2)?;
/// let output = decode_raw(&buffer, 2);
/// assert_eq!(&input, &str::from_utf8(&output)?);
/// # Ok(())
/// # }
/// ```
pub fn decode_raw<'a>(buffer: &'a [u8], size: usize, step: usize) -> (&'a [u8], Vec<u8>) {
    debug_assert_step_size!(step);
    let index_step = u8::BITS as usize / step;
    let mut out = Vec::<u8>::new();
    for index in (0..buffer.len()).step_by(index_step).take(size) {
        if let Some(byte) = decode_byte(&buffer[index..index + index_step], step) {
            out.push(byte);
        } else {
            break;
        }
    }
    (&buffer[(size * index_step)..], out)
}

pub fn encode<'a>(
    buffer: &'a mut [u8],
    data: &[u8],
    step: usize,
) -> Result<&'a mut [u8], BufferSizeError> {
    debug_assert_step_size!(step);
    let size = data.len().to_be_bytes();
    let buffer = encode_raw(buffer, &size, step)?;
    Ok(encode_raw(buffer, data, step)?)
}

pub fn decode(buffer: &[u8], step: usize) -> Result<Vec<u8>, BufferSizeError> {
    debug_assert_step_size!(step);
    let (buffer, size) = decode_raw(buffer, size_of::<usize>(), step);
    if size.len() != size_of::<usize>() {
        return Err(BufferSizeError {
            missing: size.len(),
        });
    }
    let size = usize::from_be_bytes(size.try_into().unwrap());
    let (_, data) = decode_raw(buffer, size, step);
    if data.len() != size {
        Err(BufferSizeError {
            missing: size - data.len(),
        })
    } else {
        Ok(data)
    }
}
