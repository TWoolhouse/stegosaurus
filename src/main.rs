use std::{error::Error, fs::File, io::BufReader, str};

const STEGO: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    assert_eq!(8usize % STEGO, 0);

    let file = File::open("img/Classic_Rainbow_Flag.png")?;
    let file = BufReader::new(file);

    let decoder = png::Decoder::new(file);
    let mut reader = decoder.read_info().unwrap();
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf)?;
    buf.truncate(info.buffer_size());

    let mut buf = vec![0; 100];

    let msg = "Hello World";
    stegosaurus::encode_raw(&mut buf, msg.as_bytes(), STEGO)?;

    let out = stegosaurus::decode_raw(&buf, STEGO);
    println!("Raw: {:?}", &out[..msg.len()]);
    let message = str::from_utf8(&out[..msg.len()]).unwrap();
    println!("Output: {message}");

    Ok(())
}
