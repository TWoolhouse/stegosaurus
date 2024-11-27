pub mod byte;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Buffer is too small: Buffer is {actual} bytes, but {required} bytes is required.")]
    BufferTooSmall { actual: usize, required: usize },
}
