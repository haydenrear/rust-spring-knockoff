use std::fs;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
use std::path::{Path, PathBuf};
use knockoff_helper::{program_src, project_directory};
use optional::FlatMapResult;

pub fn copy_dir(src_path: &Path, dst_path: &Path) -> std::io::Result<()> {
    for entry in std::fs::read_dir(src_path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let entry_name = entry.file_name();
        let dst_entry_path = dst_path.join(entry_name);
        if entry_path.is_file() {
            fs::copy(entry_path, dst_entry_path)?;
        } else {
            fs::create_dir_all(&dst_entry_path)?;
            copy_dir(&entry_path, &dst_entry_path)?;
        }
    }
    Ok(())
}

pub fn read_dir_to_file(path: &PathBuf) -> Result<String, std::io::Error> {
    File::open(path)
        .flat_map_res(|mut f| {
            let mut cargo_str = "".to_string();
            f.read_to_string(&mut cargo_str)
                .map(|r| cargo_str)
        })
}


pub fn open_file_read(path: &PathBuf) -> Result<File, std::io::Error> {
    File::options().read(true).open(path)
}

pub fn rewrite_file(cargo_path: &Path, output: String) -> Result<(), std::io::Error> {
    File::options().write(true).truncate(true).create(true).open(cargo_path)
        .flat_map_res(|mut file| file.write_all(output.as_bytes()))
}

#[test]
fn test_write() {
    let base_dir = Path::new(project_directory!()).join("codegen_utils").join("test_resources");
    let doesnt_exist = base_dir.join("doesnt.txt");
    let del = std::fs::remove_file(&doesnt_exist);
    assert!(!&doesnt_exist.exists());
    rewrite_file(&doesnt_exist, "this is a test".to_string());
    assert!(base_dir.exists());
    let read = read_dir_to_file(&doesnt_exist);
    assert!(read.is_ok());
    assert_eq!(read.unwrap(), "this is a test".to_string());

    let exists = base_dir.join("exists.txt");
    let read = read_dir_to_file(&exists);
    assert!(read.is_ok());
    assert_eq!(read.unwrap(), "hello".to_string());

    rewrite_file(&exists, "this is another test".to_string());
    let read = read_dir_to_file(&exists);
    assert!(read.is_ok());
    assert_eq!(read.unwrap(), "this is another test".to_string());

    rewrite_file(&exists, "hello".to_string());
    let read = read_dir_to_file(&exists);
    assert!(read.is_ok());
    assert_eq!(read.unwrap(), "hello".to_string());
}