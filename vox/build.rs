use std::{io, process::Command};

fn dashboard_build_command() -> Command {
  #[cfg(windows)]
  {
    let mut command = Command::new("cmd.exe");
    command.args(["/d", "/s", "/c", "pnpm run build"]);
    command
  }

  #[cfg(not(windows))]
  {
    let mut command = Command::new("pnpm");
    command.args(["run", "build"]);
    command
  }
}

fn main() -> io::Result<()> {
  println!("cargo:rerun-if-changed=../dashboard/src");
  println!("cargo:rerun-if-changed=../dashboard/public");
  println!("cargo:rerun-if-changed=../dashboard/package.json");
  println!("cargo:rerun-if-changed=../dashboard/pnpm-lock.yaml");

  let status =
    dashboard_build_command().current_dir("../dashboard").status().map_err(|err| {
      io::Error::new(
        err.kind(),
        format!("failed to launch pnpm for dashboard build: {err}"),
      )
    })?;
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
