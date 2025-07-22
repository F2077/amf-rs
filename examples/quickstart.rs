use amf_rs::amf0::nested::Amf0TypedValue;
use amf_rs::traits::Unmarshall;
use std::path::PathBuf;
use std::{
    env,
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // 生成FLV文件
    let mut flv_path = manifest_dir.clone();
    flv_path.push("examples");
    flv_path.push("test.flv");

    // Extract script data from the FLV file
    let script_data = extract_script_data(flv_path.as_path().to_str().unwrap())?;

    // Parse metadata from the script data
    let metadata = parse_metadata(&script_data)?;

    println!("Parsed Metadata: {}", metadata);

    Ok(())
}

fn extract_script_data(flv_path: &str) -> io::Result<Vec<u8>> {
    let mut input_file = BufReader::new(File::open(flv_path)?);
    let mut header = [0u8; 9];
    input_file.read_exact(&mut header)?;
    if &header[0..3] != b"FLV" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid FLV file",
        ));
    }

    // Skip FLV header and first PreviousTagSize
    input_file.seek(SeekFrom::Start(13))?;

    loop {
        let mut tag_header = [0u8; 11];
        if input_file.read_exact(&mut tag_header).is_err() {
            break;
        }

        let tag_type = tag_header[0];
        let data_size = u32::from_be_bytes([0, tag_header[1], tag_header[2], tag_header[3]]);

        if tag_type == 18 {
            // Script Data tag found
            let mut tag_data = vec![0u8; data_size as usize];
            input_file.read_exact(&mut tag_data)?;
            return Ok(tag_data);
        } else {
            // Skip non-script data tags
            input_file.seek(SeekFrom::Current(data_size as i64 + 4))?;
        }
    }

    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "ScriptData Tag not found",
    ))
}

fn parse_metadata(script_data: &[u8]) -> Result<String, Box<dyn std::error::Error>> {
    let mut offset = 0;
    let mut metadata = String::new();

    while offset < script_data.len() {
        let (value, bytes_read) = Amf0TypedValue::unmarshall(&script_data[offset..])?;
        let value_str = format!("{}", value);
        if value_str != "\"onMetaData\"" {
            metadata.push_str(&value_str);
        }
        offset += bytes_read;
    }

    Ok(metadata)
}
