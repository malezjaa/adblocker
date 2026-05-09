use std::io;

fn main() -> io::Result<()> {
  #[cfg(all(windows, not(debug_assertions)))]
  {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    res.compile()?;
  }
  Ok(())
}