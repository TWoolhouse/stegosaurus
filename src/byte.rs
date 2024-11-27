use crate::Error;
use std::mem::size_of;

/// Ensure the step size is valid
/// Step must be [1, 8] and a factor of 8
macro_rules! debug_assert_step_size {
    ( $step:ident ) => {
        debug_assert!(0 < $step && $step < u8::BITS as usize);
        debug_assert_eq!(u8::BITS as usize % $step, 0);
    };
}

/// Compute the number of bytes required to encode a single byte using the given step size
fn bytes_per_byte(step: usize) -> usize {
    debug_assert_step_size!(step);
    u8::BITS as usize / step
}

/// Encodes `data` into the `buffer` using the `step` least significant bits
///
/// # Arguments
///
/// * `buffer` - The buffer to encode the data into
/// * `data` - The data to encode
/// * `step` - The number of least significant bits to use
/// # Returns
/// The unused portion of the buffer
/// # Unsafe
/// This function is unsafe because it does not check if the buffer is large enough to hold the data
/// ```
unsafe fn encode_raw_unsafe<'a>(buffer: &'a mut [u8], data: &[u8], step: usize) -> &'a mut [u8] {
    debug_assert_step_size!(step);
    let space = bytes_per_byte(step);
    debug_assert!(
        buffer.len() >= (data.len() * space),
        "Buffer is too small to encode the data"
    );
    let mut it = buffer.iter_mut();
    for byte_in in data {
        let mut bit_read: u8 = 0;
        for slot in (&mut it).take(space) {
            for bit_write in 0..step {
                let insert = (byte_in & (1 << bit_read)) >> bit_read;
                let old = (*slot) & (!(1 << bit_write));
                *slot = old | (insert << bit_write);
                bit_read += 1;
            }
        }
    }
    &mut buffer[data.len() * space..]
}

/// Decodes a single byte from the `buffer` using the `step` least significant bits
/// # Arguments
/// * `buffer` - The buffer to decode the data from
/// * `step` - The number of least significant bits to use
/// # Returns
/// The decoded byte or `None` if the buffer is too small
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
    if bit == u8::BITS as u8 {
        Some(current.reverse_bits())
    } else {
        None
    }
}

/// Encodes `data` into the `buffer` using the `step` least significant bits
/// # Examples
/// ```rust
/// # use stegosaurus::{Error, byte::encode_raw};
/// # fn main() -> Result<(), Error> {
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
) -> Result<&'a mut [u8], Error> {
    debug_assert_step_size!(step);
    if buffer.len() < (data.len() * bytes_per_byte(step)) {
        return Err(Error::BufferTooSmall {
            actual: buffer.len(),
            required: data.len() * step,
        });
    }

    // SAFETY: We just checked that the buffer is large enough
    Ok(unsafe { encode_raw_unsafe(buffer, data, step) })
}

/// Decodes the data from the `buffer` from the `step` least significant bits
/// # Examples
/// ```rust
/// # use std::{error::Error, str};
/// # use stegosaurus::byte::{encode_raw, decode_raw};
/// # fn main() -> Result<(), Box<dyn Error>> {
/// let input = "Hello World";
/// let mut buffer = vec![0; 44];
/// encode_raw(&mut buffer, &input.as_bytes(), 2)?;
/// let output = decode_raw(&buffer, input.len(), 2);
/// assert_eq!(&input, &str::from_utf8(&output.1)?);
/// # Ok(())
/// # }
/// ```
pub fn decode_raw<'a>(buffer: &'a [u8], size: usize, step: usize) -> (&'a [u8], Vec<u8>) {
    debug_assert_step_size!(step);
    let bpb = bytes_per_byte(step);
    debug_assert!(
        buffer.len() >= size * bpb,
        "Buffer is too small to hold the requested amount of data"
    );
    let mut out = Vec::<u8>::new();
    for index in (0..buffer.len()).step_by(bpb).take(size) {
        if let Some(byte) = decode_byte(&buffer[index..index + bpb], step) {
            out.push(byte);
        } else {
            break;
        }
    }
    (&buffer[(size * bpb)..], out)
}

pub fn encode<'a>(buffer: &'a mut [u8], data: &[u8], step: usize) -> Result<&'a mut [u8], Error> {
    debug_assert_step_size!(step);
    let size = data.len().to_be_bytes();
    let buffer = encode_raw(buffer, &size, step)?;
    Ok(encode_raw(buffer, data, step)?)
}

pub fn decode(buffer: &[u8], step: usize) -> Result<Vec<u8>, Error> {
    debug_assert_step_size!(step);
    let (buffer, size) = decode_raw(buffer, size_of::<usize>(), step);
    if size.len() != size_of::<usize>() {
        return Err(Error::BufferTooSmall {
            actual: size.len(),
            required: size_of::<usize>(),
        });
    }
    let size = usize::from_be_bytes(size.try_into().unwrap());
    let (_, data) = decode_raw(buffer, size, step);
    if data.len() != size {
        Err(Error::BufferTooSmall {
            actual: buffer.len(),
            required: size,
        })
    } else {
        Ok(data)
    }
}

mod test {

    #[test]
    fn encode() {
        let msg = "Hi"; // 0b01001000 0b01101001
        let mut buffer = vec![0; 4];
        unsafe { super::encode_raw_unsafe(&mut buffer, &msg.as_bytes(), 4) };
        assert_eq!(buffer, vec![0b1000, 0b0100, 0b1001, 0b0110]);
    }

    #[test]
    fn decode_byte() {
        let buffer = [0b1000, 0b0100];
        let byte = super::decode_byte(&buffer, 4).unwrap();
        assert_eq!(byte, 0b01001000); // 'H'
    }

    #[test]
    fn encode_decode() {
        const STEP: usize = 2;
        let msg = "Hello World";
        let mut buffer = vec![0; 88];
        super::encode_raw(&mut buffer, &msg.as_bytes(), STEP).unwrap();
        let decoded = super::decode_raw(&buffer, msg.len(), STEP);
        assert_eq!(msg.as_bytes(), decoded.1.as_slice());
    }
}
