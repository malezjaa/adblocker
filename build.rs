use std::io;
use std::process::Command;

fn main() -> io::Result<()> {
  Command::new("pnpm").arg("vite build").current_dir("dashboard").status()?;

  #[cfg(all(windows, not(debug_assertions)))]
  {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    res.compile()?;
  }
  Ok(())
}
