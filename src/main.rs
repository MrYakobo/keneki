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

impl Drop for TempFile {
    fn drop(&mut self) {
        // Attempt to remove the file when the TempFile is dropped
        if let Err(e) = std::fs::remove_file(&self.path) {
            eprintln!("Failed to remove temporary file: {}", e);
        }
    }
}

// hopefully it is there :)
const WRAPPER: &[u8] = include_bytes!("/usr/bin/unshare");
//const WRAPPER: &[u8] = include_bytes!("/opt/homebrew/bin/bash");

// Embed user-specified binary using env var from build.rs
const BINARY: &[u8] = include_bytes!(env!("EMBEDDED_BINARY"));

// fn bytes_to_tmpfile(bytes: &[u8], path: &str) {
//     // Extract unshare
//     let mut file = File::create(path).expect("Failed to create unshare file");
//     file.write_all(bytes)
//         .expect("Failed to write bytes to temporary file");

//     std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
//         .expect("Failed to set execute permissions for unshare");
// }

fn main() {
    let mut unshare_file = TempFile::new().unwrap();
    unshare_file.write(WRAPPER).unwrap();
    unshare_file.set_permissions(0o755).unwrap();
    let unshare_path = unshare_file.path();

    // let unshare_path = "/tmp/unshare_extracted";
    // let binary_path = "/tmp/extracted_binary";
    let mut binary_file = TempFile::new().unwrap();
    binary_file.set_permissions(0o755).unwrap();
    binary_file.write(BINARY).unwrap();
    let binary_path = binary_file.path();

    // // Extract unshare
    // bytes_to_tmpfile(WRAPPER, unshare_path);

    // // Extract user binary
    // bytes_to_tmpfile(BINARY, binary_path);

    let args = std::env::args().skip(1).collect::<Vec<String>>();

    println!("{:?} -c {:?} {:?}", unshare_path, binary_path, args);

    let exit_code = Command::new(unshare_path)
        .arg("-n")
        .arg("-r")
        .arg(binary_path)
        .args(&args)
        .exec();

    if exit_code.raw_os_error().unwrap() != 0 {
        eprintln!("== keneki child process error: \n{}", exit_code);
    }
}
