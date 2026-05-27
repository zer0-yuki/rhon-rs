use std::{env, fmt, fs, io, path};

use colored::Colorize;

use crate::common::diff::{DiffOp, diff};

pub const SNAPSHOT_UPDATE_VAR: &str = "SNAPSHOT_UPDATE";

pub struct DisplayToDebug<T>(pub T);

impl<T: std::fmt::Display> std::fmt::Debug for DisplayToDebug<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

#[macro_export]
macro_rules! assert_snapshot {
    ($($name:ident = $val:expr),* $(,)?) => {
        let mut output = std::string::String::new();
        {
            use std::fmt::Write;
            $(
                let _ = writeln!(output, "{} = {:#?}", stringify!($name), &$val);
            )*
        }
        $crate::assert_snapshot!($crate::common::assert::snapshot::DisplayToDebug(output));
    };

    ($val:expr) => {
        $crate::common::assert::snapshot::__private::assert_snapshot(
            $val,
            &$crate::_snapshot_path!(),
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! _function_name {
    () => {{
        fn f() {}
        fn type_name_of_val<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let mut name = type_name_of_val(f).strip_suffix("::f").unwrap_or("");
        while let Some(rest) = name.strip_suffix("::{{closure}}") {
            name = rest;
        }
        name
    }};
}

#[doc(hidden)]
#[macro_export]
macro_rules! _snapshot_path {
    () => {{
        let file = file!();
        let base = std::path::Path::new(file);
        let parent = base.parent().unwrap_or_else(|| std::path::Path::new("."));
        let filename = format!(
            "{}.snap",
            $crate::common::assert::snapshot::__private::gen_snapshot_name(
                $crate::_function_name!()
            )
        );
        parent.join("snapshots").join(filename)
    }};
}

pub mod __private {
    use super::*;
    use std::{cell::RefCell, fmt};

    pub struct SnapshotNameHandler {
        fn_name: String,
        order: usize,
    }

    impl SnapshotNameHandler {
        const fn new() -> Self {
            Self {
                fn_name: String::new(),
                order: 0,
            }
        }

        pub fn gen_snapshot_name(&mut self, fn_name: &str) -> String {
            if fn_name == self.fn_name {
                self.order += 1;
            } else {
                self.fn_name = fn_name.into();
                self.order = 0;
            }
            format!("{}-{}", fn_name, self.order)
        }
    }

    #[doc(hidden)]
    thread_local! {
        pub static HANDLER: RefCell<SnapshotNameHandler> = RefCell::new(SnapshotNameHandler::new());
    }

    #[doc(hidden)]
    pub fn gen_snapshot_name(fn_name: &str) -> String {
        HANDLER.with(|s| s.borrow_mut().gen_snapshot_name(fn_name))
    }

    #[doc(hidden)]
    pub fn assert_snapshot<T: fmt::Debug, P: AsRef<path::Path>>(val: T, filename: P) {
        if let Err(e) = try_assert_snapshot(val, filename) {
            panic!("Failed to read file: {}", e)
        }
    }
}

fn try_assert_snapshot<T: fmt::Debug, P: AsRef<path::Path>>(val: T, filename: P) -> io::Result<()> {
    let filename = filename.as_ref();
    let actual = format!("{:#?}", val);

    let expected = match fs::read_to_string(filename) {
        Ok(s) => Some(s),
        Err(e) if e.kind() == io::ErrorKind::NotFound => None,
        Err(e) => return Err(e),
    };

    if env::var(SNAPSHOT_UPDATE_VAR).is_err() {
        if let Some(expected) = expected {
            if actual == expected {
                return Ok(());
            } else {
                panic!(
                    "\nSnapshot {} changed.\nUse `cargo snapshot` to handle this issue.\n",
                    filename.display().to_string().underline()
                );
            }
        }

        panic!(
            "\nCould not find snapshot {}.\nUse `cargo snapshot` to handle this issue.\n",
            filename.display().to_string().underline()
        );
    } else {
        if let Some(expected) = expected {
            if actual == expected {
                eprintln!(
                    "\nSnapshot {} unchanged.",
                    filename.display().to_string().underline()
                );
                return Ok(());
            } else {
                eprintln!(
                    "\nSnapshot {} changed:",
                    filename.display().to_string().underline()
                );
                show_diff(&expected, &actual, filename);
            }
        } else {
            eprintln!(
                "\nSnapshot {} does not exist, it will be created.",
                filename.display().to_string().underline()
            );
            show_diff("", &actual, filename);
        }

        use io::Write;

        let mut stdin = io::stdin().lock();

        loop {
            eprint!("Do you want to apply changes to this snapshot? (Y/n) ");
            io::stderr().flush()?;

            let mut input = String::new();
            use io::BufRead;
            stdin.read_line(&mut input).expect("Unable to read input");
            let input = input.trim();

            match input {
                "y" | "Y" | "" => break,
                "n" | "N" => return Ok(()),
                _ => {
                    eprintln!("`{}` is not a valid choice.", input);
                    continue;
                }
            }
        }

        if let Some(parent) = path::Path::new(filename).parent() {
            fs::create_dir_all(parent)?;
        }

        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(filename)?;
        write!(&file, "{}", actual)?;
        eprintln!(
            "Suceeded to write to snapshot {}.",
            filename.display().to_string().underline()
        );
    }

    Ok(())
}

fn show_diff<P: AsRef<path::Path>>(old: &str, new: &str, filename: P) {
    let old_lines: Vec<_> = old.split("\n").collect();
    let new_lines: Vec<_> = new.split("\n").collect();

    let diff_ops = diff(&old_lines, &new_lines);
    eprintln!("┏{}", "━".repeat(40));
    // Safe to use `unwrap` because filename generated by macros.
    // TODO: should we pass other info instead of filename?
    // That means we may do changes in macros.
    // And we may also need meta comments in snap files in the future.
    eprintln!("┃ {}", filename.as_ref().file_name().unwrap().display());
    eprintln!("┣{}", "━".repeat(40));
    for op in diff_ops {
        let line = match op {
            DiffOp::Keep(s) => format!("  {}", s).white(),
            DiffOp::Insert(s) => format!("+ {}", s).green(),
            DiffOp::Delete(s) => format!("- {}", s).red(),
        };
        eprintln!("┃ {}", line);
    }
    eprintln!("┗{}", "━".repeat(40));
}
