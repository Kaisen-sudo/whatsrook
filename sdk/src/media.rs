//! Media encoding and data URL helper utilities.

#![allow(
    unknown_lints,
    clippy::manual_div_ceil,
    clippy::chunks_exact_to_as_chunks,
    clippy::manual_is_multiple_of
)]

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

/// Standard RFC 4648 Base64 alphabet.
const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// Error returned when decoding an invalid Base64 sequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Base64Error {
    /// Found an invalid character not present in the RFC 4648 alphabet.
    InvalidByte(char),
    /// Input length is not a valid Base64 length or padding is incorrect.
    InvalidLength,
}

impl fmt::Display for Base64Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Base64Error::InvalidByte(c) => {
                write!(f, "invalid character in base64 payload: {:?}", c)
            }
            Base64Error::InvalidLength => write!(f, "invalid base64 payload length or padding"),
        }
    }
}

impl std::error::Error for Base64Error {}

/// Encodes arbitrary bytes into a standard RFC 4648 Base64 string with `=` padding.
///
/// # Example
///
/// ```
/// use whatsrook_sdk::media::encode_base64;
/// assert_eq!(encode_base64(b"hello world"), "aGVsbG8gd29ybGQ=");
/// ```
pub fn encode_base64(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    let mut chunks = bytes.chunks_exact(3);

    for chunk in chunks.by_ref() {
        let b0 = chunk[0];
        let b1 = chunk[1];
        let b2 = chunk[2];

        out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
        out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(BASE64_ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        out.push(BASE64_ALPHABET[(b2 & 0x3f) as usize] as char);
    }

    let rem = chunks.remainder();
    match rem.len() {
        1 => {
            let b0 = rem[0];
            out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
            out.push(BASE64_ALPHABET[((b0 & 0x03) << 4) as usize] as char);
            out.push('=');
            out.push('=');
        }
        2 => {
            let b0 = rem[0];
            let b1 = rem[1];
            out.push(BASE64_ALPHABET[(b0 >> 2) as usize] as char);
            out.push(BASE64_ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
            out.push(BASE64_ALPHABET[((b1 & 0x0f) << 2) as usize] as char);
            out.push('=');
        }
        _ => {}
    }

    out
}

/// Decodes an RFC 4648 Base64 string into bytes.
///
/// Ignores whitespace characters (newlines, carriage returns, spaces).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::media::decode_base64;
/// let bytes = decode_base64("aGVsbG8gd29ybGQ=").unwrap();
/// assert_eq!(bytes, b"hello world");
/// ```
pub fn decode_base64(input: &str) -> Result<Vec<u8>, Base64Error> {
    let clean: Vec<u8> = input
        .bytes()
        .filter(|&b| !b.is_ascii_whitespace())
        .collect();

    if clean.is_empty() {
        return Ok(Vec::new());
    }

    if !clean.len().is_multiple_of(4) {
        return Err(Base64Error::InvalidLength);
    }

    let mut out = Vec::with_capacity((clean.len() / 4) * 3);

    for chunk in clean.chunks_exact(4) {
        let mut vals = [0u8; 4];
        let mut pad_count = 0;

        for (i, &b) in chunk.iter().enumerate() {
            if b == b'=' {
                pad_count += 1;
                vals[i] = 0;
            } else {
                if pad_count > 0 {
                    return Err(Base64Error::InvalidLength);
                }
                vals[i] = match b {
                    b'A'..=b'Z' => b - b'A',
                    b'a'..=b'z' => b - b'a' + 26,
                    b'0'..=b'9' => b - b'0' + 52,
                    b'+' => 62,
                    b'/' => 63,
                    _ => return Err(Base64Error::InvalidByte(b as char)),
                };
            }
        }

        let n = ((vals[0] as u32) << 18)
            | ((vals[1] as u32) << 12)
            | ((vals[2] as u32) << 6)
            | (vals[3] as u32);

        out.push((n >> 16) as u8);
        if pad_count < 2 {
            out.push((n >> 8) as u8);
        }
        if pad_count < 1 {
            out.push(n as u8);
        }
    }

    Ok(out)
}

/// Formats a base64 payload into a Data URL (`data:<mimetype>;base64,<data>`).
///
/// # Example
///
/// ```
/// use whatsrook_sdk::media::to_data_url;
/// let url = to_data_url("image/png", "aGVsbG8=");
/// assert_eq!(url, "data:image/png;base64,aGVsbG8=");
/// ```
pub fn to_data_url(mimetype: &str, base64_data: &str) -> String {
    format!("data:{};base64,{}", mimetype, base64_data)
}

/// Reads a local file from disk and returns its contents encoded as Base64.
///
/// # Example
///
/// ```no_run
/// use whatsrook_sdk::media::read_file_as_base64;
/// let b64 = read_file_as_base64("image.png").unwrap();
/// ```
pub fn read_file_as_base64(path: impl AsRef<Path>) -> io::Result<String> {
    let bytes = fs::read(path)?;
    Ok(encode_base64(&bytes))
}

/// Reads a local file from disk and returns a complete Data URL (`data:<mimetype>;base64,...`).
///
/// # Example
///
/// ```no_run
/// use whatsrook_sdk::media::read_file_as_data_url;
/// let data_url = read_file_as_data_url("voice.ogg", "audio/ogg; codecs=opus").unwrap();
/// ```
pub fn read_file_as_data_url(path: impl AsRef<Path>, mimetype: &str) -> io::Result<String> {
    let b64 = read_file_as_base64(path)?;
    Ok(to_data_url(mimetype, &b64))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base64_roundtrip() {
        let test_cases: &[&[u8]] = &[
            b"",
            b"f",
            b"fo",
            b"foo",
            b"foob",
            b"fooba",
            b"foobar",
            b"Rust in WhatsApp plugins is awesome!",
        ];

        for &tc in test_cases {
            let enc = encode_base64(tc);
            let dec = decode_base64(&enc).expect("decode failed");
            assert_eq!(dec, tc, "failed on {:?}", tc);
        }
    }

    #[test]
    fn test_data_url() {
        assert_eq!(
            to_data_url("image/webp", "XYZ123"),
            "data:image/webp;base64,XYZ123"
        );
    }
}
