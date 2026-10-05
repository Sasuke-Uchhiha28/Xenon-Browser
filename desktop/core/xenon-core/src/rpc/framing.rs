//! Line framing for the stdio RPC channel: read one message per line,
//! enforce the 1 MB limit while reading (so an oversized line cannot
//! balloon memory), and keep the stream in sync when rejecting one.

use crate::{CoreError, Result};
use std::io::BufRead;

/// Line reader with a size cap.
pub struct FrameReader<R: BufRead> {
    inner: R,
    max_line_bytes: usize,
}

impl<R: BufRead> FrameReader<R> {
    /// Wrap a reader with the given line limit.
    pub fn new(inner: R, max_line_bytes: usize) -> FrameReader<R> {
        FrameReader {
            inner,
            max_line_bytes,
        }
    }

    /// Read one line (without its newline). `Ok(None)` at a clean EOF.
    /// An oversized line drains the rest of the line from the stream and
    /// fails with code -32000, so the next read starts on the next line.
    pub fn read_message(&mut self) -> Result<Option<String>> {
        let mut buf: Vec<u8> = Vec::new();
        let mut over_limit = false;
        loop {
            let outcome = {
                let available = self.inner.fill_buf()?;
                if available.is_empty() {
                    Scan::Eof
                } else if let Some(pos) = available.iter().position(|&byte| byte == b'\n') {
                    if !over_limit {
                        buf.extend_from_slice(&available[..pos]);
                    }
                    self.inner.consume(pos + 1);
                    Scan::Line
                } else {
                    if over_limit {
                        let len = available.len();
                        self.inner.consume(len);
                    } else {
                        buf.extend_from_slice(available);
                        let len = available.len();
                        self.inner.consume(len);
                        if buf.len() > self.max_line_bytes {
                            over_limit = true;
                            buf = Vec::new();
                        }
                    }
                    Scan::Continue
                }
            };
            match outcome {
                Scan::Line => {
                    if buf.len() > self.max_line_bytes {
                        return Err(CoreError::Rpc {
                            code: crate::rpc::codes::TOO_LARGE,
                            message: format!("message exceeds {} bytes", self.max_line_bytes),
                        });
                    }
                    while buf.last() == Some(&b'\r') {
                        buf.pop();
                    }
                    let line = String::from_utf8(buf).map_err(|_| CoreError::Rpc {
                        code: crate::rpc::codes::PARSE,
                        message: "line is not valid UTF-8".into(),
                    })?;
                    return Ok(Some(line));
                }
                Scan::Eof => {
                    return if over_limit {
                        Err(CoreError::Rpc {
                            code: crate::rpc::codes::TOO_LARGE,
                            message: "message exceeds the line limit".into(),
                        })
                    } else if buf.is_empty() {
                        Ok(None)
                    } else {
                        // Last line without a trailing newline still counts.
                        while buf.last() == Some(&b'\r') {
                            buf.pop();
                        }
                        let line = String::from_utf8(buf).map_err(|_| CoreError::Rpc {
                            code: crate::rpc::codes::PARSE,
                            message: "line is not valid UTF-8".into(),
                        })?;
                        Ok(Some(line))
                    };
                }
                Scan::Continue => continue,
            }
        }
    }
}

enum Scan {
    Line,
    Eof,
    Continue,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufReader;

    /// Read every message, failing the test on any error (including
    /// size-limit rejections, which the dedicated test below drives
    /// directly).
    fn lines(input: &[u8]) -> Vec<String> {
        let mut reader = FrameReader::new(BufReader::new(input), 16);
        let mut out = Vec::new();
        while let Some(line) = reader.read_message().expect("unexpected error") {
            out.push(line);
        }
        out
    }

    #[test]
    fn reads_lines_and_stops_at_eof() {
        assert_eq!(
            lines(b"one\ntwo\n"),
            vec!["one".to_string(), "two".to_string()]
        );
    }

    #[test]
    fn handles_missing_trailing_newline() {
        assert_eq!(
            lines(b"one\ntail"),
            vec!["one".to_string(), "tail".to_string()]
        );
    }

    #[test]
    fn empty_input_is_clean_eof() {
        assert!(lines(b"").is_empty());
        assert_eq!(lines(b"\n\n"), vec!["".to_string(), "".to_string()]);
    }

    #[test]
    fn crlf_is_trimmed() {
        assert_eq!(lines(b"ping\r\n"), vec!["ping".to_string()]);
    }

    #[test]
    fn oversized_line_is_rejected_and_stream_stays_in_sync() {
        let mut reader = FrameReader::new(
            BufReader::new(b"short\n0123456789abcdef0123456789abcdef\nnext\n".as_slice()),
            16,
        );
        assert_eq!(
            reader.read_message().expect("first").expect("some"),
            "short"
        );
        match reader.read_message() {
            Err(CoreError::Rpc { code, .. }) => {
                assert_eq!(code, crate::rpc::codes::TOO_LARGE);
            }
            other => panic!("expected a too-large error, got {other:?}"),
        }
        assert_eq!(
            reader.read_message().expect("third").expect("some"),
            "next",
            "framing must stay in sync after an oversized line"
        );
    }
}
