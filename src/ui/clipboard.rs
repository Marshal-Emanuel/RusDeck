use std::io::Write;
use std::process::{Command, Stdio};

pub fn copy_to_clipboard(text: &str) -> bool {
    // 1. Try wl-copy if WAYLAND_DISPLAY is set
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                if stdin.write_all(text.as_bytes()).is_ok() {
                    drop(stdin);
                    if let Ok(status) = child.wait() {
                        if status.success() {
                            return true;
                        }
                    }
                }
            }
        }
    }

    // 2. Try xsel
    if let Ok(mut child) = Command::new("xsel")
        .args(&["--clipboard", "--input"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if let Ok(status) = child.wait() {
                    if status.success() {
                        return true;
                    }
                }
            }
        }
    }

    // 3. Try xclip
    if let Ok(mut child) = Command::new("xclip")
        .args(&["-selection", "clipboard", "-i"])
        .stdin(Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            if stdin.write_all(text.as_bytes()).is_ok() {
                drop(stdin);
                if let Ok(status) = child.wait() {
                    if status.success() {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn paste_from_clipboard() -> Option<String> {
    // 1. Try wl-paste if WAYLAND_DISPLAY is set
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        if let Ok(output) = Command::new("wl-paste").output() {
            if output.status.success() {
                if let Ok(text) = String::from_utf8(output.stdout) {
                    return Some(text);
                }
            }
        }
    }

    // 2. Try xsel
    if let Ok(output) = Command::new("xsel").args(&["--clipboard", "--output"]).output() {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                return Some(text);
            }
        }
    }

    // 3. Try xclip
    if let Ok(output) = Command::new("xclip").args(&["-selection", "clipboard", "-o"]).output() {
        if output.status.success() {
            if let Ok(text) = String::from_utf8(output.stdout) {
                return Some(text);
            }
        }
    }

    None
}