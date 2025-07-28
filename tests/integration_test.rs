use std::{
    fs::File,
    io::{self, BufReader, Read, Seek, SeekFrom},
    process::Command,
};

mod test_setup {
    use super::*;
    use once_cell::sync::OnceCell;
    use regex::Regex;
    use std::path::PathBuf;
    use std::{env, fs};

    static SCRIPT_DATA: OnceCell<(Vec<u8>, String)> = OnceCell::new();

    pub fn setup() -> &'static (Vec<u8>, String) {
        SCRIPT_DATA.get_or_init(|| flv_metadata_generation().unwrap())
    }

    fn flv_metadata_generation() -> io::Result<(Vec<u8>, String)> {
        // 检查必要命令是否存在
        assert!(command_exists("ffmpeg"), "ffmpeg not installed");
        assert!(command_exists("flvmeta"), "flvmeta not installed");
        assert!(check_ffmpeg_version(6), "requires ffmpeg version 6.0+");

        // 获取包含 Cargo.toml 的目录的路径
        let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

        // 生成FLV文件
        let mut output_path = manifest_dir.clone();
        output_path.push("tests");
        output_path.push("output.flv");
        let status = Command::new("ffmpeg")
            .args(&[
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=320x240:rate=25",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:sample_rate=44100",
                "-t",
                "1",
                "-c:v",
                "flv",
                "-c:a",
                "libmp3lame",
                "-ar",
                "44100",
                "-ab",
                "128k",
                "-metadata",
                "title=FFmpeg AMF0 TEST",
                "-metadata",
                "author=amf-rs-tests",
                "-y",
                output_path.as_path().to_str().unwrap(),
            ])
            .status()?;
        assert!(
            status.success(),
            "FFmpeg failed to execute, exit code: {}",
            status
        );

        // 解析生成的 FLV 文件
        let mut input_file = BufReader::new(File::open(output_path.as_path())?);
        let mut header = [0u8; 9];
        input_file.read_exact(&mut header)?;
        assert_eq!(&header[0..3], b"FLV", "FLV file header verification failed");

        // 跳过 FLV Header 和第一个 PreviousTagSize
        input_file.seek(SeekFrom::Start(13))?;

        let mut tag_data = vec![];
        let mut found = false;
        loop {
            let mut tag_header = [0u8; 11];
            if input_file.read_exact(&mut tag_header).is_err() {
                break;
            }

            let tag_type = tag_header[0];
            let data_size = u32::from_be_bytes([0, tag_header[1], tag_header[2], tag_header[3]]);
            let timestamp =
                u32::from_be_bytes([tag_header[7], tag_header[4], tag_header[5], tag_header[6]]);

            if tag_type == 18 {
                // 读取完整 Tag 数据
                tag_data = vec![0u8; data_size as usize];
                input_file.read_exact(&mut tag_data)?;

                println!(
                    "Successfully extracted ScriptData Tag（Timestamp：{}）",
                    timestamp
                );
                found = true;
                break;
            } else {
                input_file.seek(SeekFrom::Current(data_size as i64 + 4))?;
            }
        }

        assert!(found, "ScriptData Tag not found");

        // 使用 flvmeta 提取 ScriptData 中数据
        let probe = Command::new("flvmeta")
            .args(&["-j", output_path.as_path().to_str().unwrap()])
            .output()?;
        assert!(probe.status.success(), "ffprobe failed");

        let json_data = String::from_utf8_lossy(&probe.stdout)
            .replace("\n", "")
            .replace("\r", "");

        // 清理FLV文件
        if output_path.exists() {
            fs::remove_file(output_path.as_path())?;
        }

        Ok((tag_data, json_data))
    }

    fn check_ffmpeg_version(min_major: u32) -> bool {
        let output = Command::new("ffmpeg")
            .arg("-version")
            .output()
            .expect("FFmpeg execution failed");

        let version_str = String::from_utf8_lossy(&output.stdout);
        let re = Regex::new(r"ffmpeg version (\d+)\.\d+\.\d+").unwrap();
        re.captures(&version_str)
            .and_then(|c| c[1].parse::<u32>().ok())
            .map(|major| major >= min_major)
            .unwrap_or(false)
    }

    fn command_exists(cmd: &str) -> bool {
        #[cfg(target_os = "windows")]
        {
            Command::new("where")
                .arg(cmd)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
        #[cfg(not(target_os = "windows"))]
        {
            Command::new("sh")
                .arg("-c")
                .arg(format!("command -v {}", cmd))
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_setup;
    use amf_rs::amf0::nested::Amf0TypedValue;
    use amf_rs::traits::Unmarshall;

    #[test]
    fn test_amf_rs() {
        let test_case = test_setup::setup();
        let buf = test_case.0.as_slice();

        let mut string_builder = String::new();
        let mut offset = 0;
        while offset < buf.len() {
            let (v, n) = Amf0TypedValue::unmarshall(&buf[offset..]).unwrap();
            let s = &format!("{}", v);
            if s != "\"onMetaData\"" {
                string_builder.push_str(s);
            }
            offset += n;
        }

        let expect = &test_case.1;
        let actual = &string_builder;
        println!("EXPECT= {}", expect);
        println!("ACTUAL= {}", actual);

        assert_eq!(expect, actual);
    }
}
