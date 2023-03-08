use std::{error::Error, fs::File, io::BufReader, str};

const STEGO: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let file = File::open("img/Classic_Rainbow_Flag.png")?;
    let file = BufReader::new(file);

    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());

    let msg = "Hello World";
    stegosaurus::encode(&mut buf, msg.as_bytes(), STEGO)?;

    let out = stegosaurus::decode(&buf, STEGO)?;
    let message = str::from_utf8(&out).unwrap();
    println!("Output: {message}");

    Ok(())
}
