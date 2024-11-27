use std::{error::Error, fs::File, io::BufReader, str};

const STEGO: usize = 2;

fn main() -> Result<(), Box<dyn Error>> {
    let msg = "Hello World how are you doing today?";
    let file = File::open("img/Classic_Rainbow_Flag.png")?;
    let mut info = (0, 0, png::ColorType::Rgba, png::BitDepth::Eight);

    let mut buf = Vec::new();
    {
        let file = BufReader::new(file);
        let decoder = png::Decoder::new(file);
        let mut reader = decoder.read_info().unwrap();
        {
            let i = reader.info();
            info.0 = i.width;
            info.1 = i.height;
            info.2 = i.color_type;
            info.3 = i.bit_depth;
        }
        buf.resize(reader.output_buffer_size(), 0);
        let frame = reader.next_frame(&mut buf)?;
        buf.truncate(frame.buffer_size());
    }

    {
        stegosaurus::byte::encode(&mut buf, msg.as_bytes(), STEGO)?;
        let mut file = File::create("img/Classic_Rainbow_Flag_stego.png")?;
        let mut encoder = png::Encoder::new(&mut file, info.0, info.1);
        encoder.set_color(info.2);
        encoder.set_depth(info.3);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&buf)?;
    }

    let out = stegosaurus::byte::decode(&buf, STEGO)?;
    let message = str::from_utf8(&out).unwrap();
    println!("Output: {message}");

    Ok(())
}
