use std::io;

fn checksum(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320_u32 & 0_u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn word(output: &mut Vec<u8>, value: u16) {
    output.extend(value.to_le_bytes());
}
fn dword(output: &mut Vec<u8>, value: u32) {
    output.extend(value.to_le_bytes());
}

pub fn zip(files: &[(String, String)]) -> io::Result<Vec<u8>> {
    if files.len() > 500 {
        return Err(io::Error::other("Too many archive entries"));
    }
    let mut output = Vec::new();
    let mut directory = Vec::new();
    let mut names = std::collections::HashSet::new();
    for (name, content) in files {
        if name.len() > 255
            || name.split('/').any(|part| {
                part.is_empty()
                    || part == "."
                    || part == ".."
                    || !part
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || b"._-".contains(&byte))
            })
            || !names.insert(name)
        {
            return Err(io::Error::other("Unsafe or duplicate archive name"));
        }
        if content.len() > 8 * 1024 * 1024 || output.len() + content.len() > 16 * 1024 * 1024 {
            return Err(io::Error::other("Archive too large"));
        }
        let offset = output.len() as u32;
        let size = content.len() as u32;
        let crc = checksum(content.as_bytes());
        dword(&mut output, 0x0403_4b50);
        for value in [20, 0x0800, 0, 0, 33] {
            word(&mut output, value);
        }
        for value in [crc, size, size] {
            dword(&mut output, value);
        }
        word(&mut output, name.len() as u16);
        word(&mut output, 0);
        output.extend(name.as_bytes());
        output.extend(content.as_bytes());
        dword(&mut directory, 0x0201_4b50);
        for value in [20, 20, 0x0800, 0, 0, 33] {
            word(&mut directory, value);
        }
        for value in [crc, size, size] {
            dword(&mut directory, value);
        }
        for value in [name.len() as u16, 0, 0, 0, 0] {
            word(&mut directory, value);
        }
        dword(&mut directory, 0);
        dword(&mut directory, offset);
        directory.extend(name.as_bytes());
    }
    let offset = output.len() as u32;
    let directory_size = directory.len() as u32;
    output.extend(directory);
    dword(&mut output, 0x0605_4b50);
    for value in [0, 0, files.len() as u16, files.len() as u16] {
        word(&mut output, value);
    }
    dword(&mut output, directory_size);
    dword(&mut output, offset);
    word(&mut output, 0);
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn archive_checks_names_and_crc() {
        assert_eq!(checksum(b"123456789"), 0xcbf4_3926);
        for name in [
            "../escape",
            "/root",
            "C:/escape",
            "path\\escape",
            "path/./file",
        ] {
            assert!(zip(&[(name.into(), "test".into())]).is_err());
        }
        let archive = zip(&[("folder/file.txt".into(), "你好".into())]).unwrap();
        assert_eq!(&archive[..4], b"PK\x03\x04");
        assert_eq!(
            &archive[archive.len() - 22..archive.len() - 18],
            b"PK\x05\x06"
        );
    }
}
