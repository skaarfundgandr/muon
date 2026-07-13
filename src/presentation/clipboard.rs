pub(crate) struct HeldClipboard(arboard::Clipboard);

impl std::fmt::Debug for HeldClipboard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Clipboard")
    }
}

pub(crate) fn copy_text_to_clipboard(
    text: &str,
    held: &mut Option<HeldClipboard>,
) -> std::result::Result<(), String> {
    if let Err(e) = copy_via_arboard(text, held) {
        if copy_via_external_cli(text).is_ok() {
            return Ok(());
        }
        return Err(e);
    }
    Ok(())
}

fn copy_via_arboard(
    text: &str,
    held: &mut Option<HeldClipboard>,
) -> std::result::Result<(), String> {
    if held.is_none() {
        match arboard::Clipboard::new() {
            Ok(cb) => *held = Some(HeldClipboard(cb)),
            Err(e) => return Err(e.to_string()),
        }
    }
    let Some(HeldClipboard(cb)) = held.as_mut() else {
        return Err("clipboard unavailable".into());
    };

    #[cfg(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    ))]
    {
        use arboard::{LinuxClipboardKind, SetExtLinux};
        cb.set()
            .clipboard(LinuxClipboardKind::Clipboard)
            .text(text.to_string())
            .map_err(|e| e.to_string())?;
        let _ = cb
            .set()
            .clipboard(LinuxClipboardKind::Primary)
            .text(text.to_string());
        Ok(())
    }

    #[cfg(not(any(
        target_os = "linux",
        target_os = "freebsd",
        target_os = "dragonfly",
        target_os = "openbsd",
        target_os = "netbsd"
    )))]
    {
        cb.set_text(text.to_string()).map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn copy_via_external_cli(text: &str) -> std::result::Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let candidates: &[(&str, &[&str])] = &[
        ("wl-copy", &[]),
        ("xclip", &["-selection", "clipboard"]),
        ("xsel", &["--clipboard", "--input"]),
    ];

    for (bin, args) in candidates {
        let Ok(mut child) = Command::new(bin)
            .args(*args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        else {
            continue;
        };
        let write_ok = child
            .stdin
            .as_mut()
            .map(|stdin| stdin.write_all(text.as_bytes()).is_ok())
            .unwrap_or(false);
        if !write_ok {
            let _ = child.kill();
            continue;
        }
        if child.wait().map(|s| s.success()).unwrap_or(false) {
            return Ok(());
        }
    }
    Err("no working clipboard backend (arboard / wl-copy / xclip / xsel)".into())
}
