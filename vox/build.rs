use std::{io, process::Command};

fn main() -> io::Result<()> {
  println!("cargo:rerun-if-changed=../dashboard/src");
  println!("cargo:rerun-if-changed=../dashboard/public");
  println!("cargo:rerun-if-changed=../dashboard/package.json");
  println!("cargo:rerun-if-changed=../dashboard/pnpm-lock.yaml");

  let status =
    Command::new("pnpm").args(["run", "build"]).current_dir("../dashboard").status()?;
  if !status.success() {
    return Err(io::Error::other("dashboard build failed"));
  }

  #[cfg(all(windows, not(debug_assertions)))]
  {
    let mut res = winres::WindowsResource::new();
    res.set_manifest_file("app.manifest");
    res.compile()?;
  }
  Ok(())
}
