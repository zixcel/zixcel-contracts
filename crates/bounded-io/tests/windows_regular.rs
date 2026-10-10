#![cfg(windows)]
use std::{fs, io};
use zixcel_bounded_io::{InputLimit, read_directory, read_regular_file, read_regular_text};
#[test]
fn regular_files_keep_windows_read_and_budget_behavior() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("日本語-input");
    for size in [0, 1, 8192, 8193] {
        fs::write(&path, vec![42; size]).unwrap();
        assert_eq!(
            read_regular_file(&path, size as u64).unwrap(),
            fs::read(&path).unwrap()
        );
        let canonical = fs::canonicalize(&path).unwrap();
        assert_eq!(
            read_regular_file(&canonical, size as u64).unwrap(),
            fs::read(&path).unwrap()
        );
        if size > 0 {
            let error = read_regular_file(&path, size as u64 - 1).unwrap_err();
            assert!(error.get_ref().unwrap().is::<InputLimit>());
        }
    }
    fs::write(&path, [255]).unwrap();
    assert_eq!(
        read_regular_text(&path, 1).unwrap_err().kind(),
        io::ErrorKind::InvalidData
    );
    assert!(read_regular_file(root.path(), 10).is_err());
    assert_eq!(
        read_regular_file(root.path().join("missing"), 10)
            .unwrap_err()
            .kind(),
        io::ErrorKind::NotFound
    );
}
#[test]
fn junction_reparse_points_are_refused_and_read_never_initializes() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("target");
    let junction = root.path().join("junction");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("file"), b"unchanged").unwrap();
    let result = std::process::Command::new("cmd.exe")
        .args(["/C", "mklink", "/J"])
        .arg(&junction)
        .arg(&target)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "junction creation failed: {:?}",
        result.stderr
    );
    assert_eq!(
        read_directory(&junction, 10).unwrap_err().kind(),
        io::ErrorKind::InvalidInput
    );
    assert!(read_regular_file(&junction, 10).is_err());
    assert_eq!(read_directory(&target, 1).unwrap().len(), 1);
    assert!(read_directory(&target, 0).is_err());
    assert!(read_directory(root.path().join("absent"), 10).is_err());
    assert!(!root.path().join("absent").exists());
    assert_eq!(fs::read(target.join("file")).unwrap(), b"unchanged");
}
