pub fn open_browser(url: &str) -> std::io::Result<()> {
    use std::process::Command;

    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(url).status()?;
    }

    #[cfg(target_os = "windows")]
    {
        Command::new("rundll32")
            .args(["url.dll,FileProtocolHandler", url])
            .status()?;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Command::new("xdg-open").arg(url).status()?;
    }

    Ok(())
}
