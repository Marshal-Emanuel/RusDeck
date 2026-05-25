use std::io::Write;

pub fn copy_to_clipboard(text: &str) -> bool {
    let cmd = if std::process::Command::new("wl-copy").stdin(std::process::Stdio::piped()).spawn().is_ok() {
        "wl-copy"
    } else {
        "xsel"
    };

    let args: &[&str] = if cmd == "xsel" {
        &["--clipboard", "--input"]
    } else {
        &[]
    };

    let mut child = match std::process::Command::new(cmd)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(_) => return false,
    };

    let mut stdin = match child.stdin.take() {
        Some(s) => s,
        None => return false,
    };

    let write_ok = stdin.write_all(text.as_bytes()).is_ok();
    drop(stdin);
    write_ok && child.wait().map(|e| e.success()).unwrap_or(false)
}