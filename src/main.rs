use std::fs::File;
use std::io::Error;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
struct TempFile {
    file: File,
    path: PathBuf,
}

impl TempFile {
    fn new() -> std::io::Result<Self> {
        // Get a unique identifier using timestamp and process id
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();

        // Create a unique filename
        let temp_dir = std::env::temp_dir();
        let filename = format!("temp_{}_{}", timestamp, pid);
        let path = temp_dir.join(filename);

        // Create and open the file
        let file = File::create(&path)?;

        Ok(TempFile { file, path })
    }

    fn set_permissions(&mut self, mode: u32) -> Result<(), Error> {
        std::fs::set_permissions(&self.path, std::fs::Permissions::from_mode(mode))
    }

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Write for TempFile {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.file.flush()
    }
}

// hopefully it is there :)
const WRAPPER: &[u8] = include_bytes!("/usr/bin/unshare");

// Embed user-specified binary using env var from build.rs
const BINARY: &[u8] = include_bytes!(env!("EMBEDDED_BINARY"));

fn main() {
    let wrapper_path = {
        let mut unshare_file = TempFile::new().unwrap();
        unshare_file.write(WRAPPER).unwrap();
        unshare_file.set_permissions(0o755).unwrap();
        unshare_file.path().clone()
    };

    let binary_path = {
        let mut binary_file = TempFile::new().unwrap();
        binary_file.set_permissions(0o755).unwrap();
        binary_file.write(BINARY).unwrap();
        binary_file.path().clone()
    };

    let args = std::env::args().skip(1).collect::<Vec<String>>();

    let exit_code = Command::new(&wrapper_path)
        .arg("--net")
        .arg("--root")
        .arg(&binary_path)
        .args(&args)
        .exec();

    if exit_code.raw_os_error().unwrap() != 0 {
        eprintln!("== keneki child process error: \n{}", exit_code);
    }

    std::fs::remove_file(wrapper_path).unwrap();
    std::fs::remove_file(binary_path).unwrap();
}
