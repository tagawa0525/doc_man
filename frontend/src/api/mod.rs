use std::fmt::Write;

pub mod approval_steps;

/// URL クエリパラメータ値のパーセントエンコード
pub fn encode_query(s: &str) -> String {
    let mut out = String::with_capacity(s.len() * 3);
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                let _ = write!(out, "%{b:02X}");
            }
        }
    }
    out
}
pub mod client;
pub mod departments;
pub mod disciplines;
pub mod distributions;
pub mod document_kinds;
pub mod document_registers;
pub mod documents;
pub mod employees;
pub mod projects;
pub mod tags;
pub mod types;
