use std::{env, process};

use rhon_rs::common::assert::snapshot::SNAPSHOT_UPDATE_VAR;

fn main() {
    println!("command: {:?}", env::args().collect::<Vec<_>>());
    let mut args: Vec<_> = env::args().skip(1).collect();

    // In direct execution, there's no problem,
    // however in cargo command, another arg `snapshot` will be added
    // after first arg.
    if args[0] == "snapshot" {
        args.remove(0);
    };

    let status = process::Command::new("cargo")
        .env(SNAPSHOT_UPDATE_VAR, "")
        .arg("test")
        .args(&args)
        .args(["--", "--nocapture", "--test-threads=1"])
        .status()
        .expect("Failed to execute `test` command.");

    process::exit(status.code().unwrap_or(1))
}
