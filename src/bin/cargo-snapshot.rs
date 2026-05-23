use std::{env, fs, path, process};

use rhon_rs::common::assert::snapshot::SNAPSHOT_UPDATE_VAR;

fn main() {
    let mut args: Vec<_> = env::args().skip(1).collect();

    // In direct execution, there's no problem,
    // however in cargo command, another arg `snapshot` will be added
    // after first arg.
    if let Some(arg) = args.first()
        && arg == "snapshot"
    {
        args.remove(0);
    };

    if let Some(arg) = args.first() {
        match arg.as_str() {
            "remove" => match remove_snapshots("./") {
                Ok(_) => process::exit(0),
                Err(e) => panic!("Failed to remove snapshots: {}", e),
            },
            "refresh" => match remove_snapshots("./") {
                Ok(_) => {
                    args.remove(0);
                }
                Err(e) => panic!("Failed to remove snapshots: {}", e),
            },
            _ => {}
        }
    }

    let status = process::Command::new("cargo")
        .env(SNAPSHOT_UPDATE_VAR, "")
        .arg("test")
        .args(&args)
        .args(["--", "--nocapture", "--test-threads=1"])
        .status()
        .expect("Failed to execute `test` command.");

    process::exit(status.code().unwrap_or(1))
}

pub fn remove_snapshots<P: AsRef<path::Path>>(root: P) -> std::io::Result<()> {
    let root = root.as_ref();
    if !root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let file_name = path.file_name().and_then(|n| n.to_str());
            if file_name == Some("snapshots") {
                for snap_entry in fs::read_dir(&path)? {
                    let snap_path = snap_entry?.path();
                    if snap_path.is_file()
                        && snap_path.extension().and_then(|e| e.to_str()) == Some("snap")
                    {
                        fs::remove_file(&snap_path)?;
                    }
                }
            } else {
                remove_snapshots(&path)?;
            }
        }
    }
    Ok(())
}
