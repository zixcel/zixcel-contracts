use super::*;
#[test]
fn disk_handles_are_required_and_device_names_refused() {
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("ordinary");
    std::fs::write(&path, b"value").unwrap();
    assert!(
        winapi_util::file::typ(open_input(&path).unwrap())
            .unwrap()
            .is_disk()
    );
    assert!(validate_path(Path::new(r"\\server\share\ordinary")).is_ok());
    assert!(validate_path(Path::new("COM10.txt")).is_ok());
    let null = OpenOptions::new().read(true).open("NUL").unwrap();
    assert!(winapi_util::file::typ(null).unwrap().is_char());
    for value in [
        "NUL",
        "nul.txt",
        "CONIN$",
        "COM1:",
        "LPT³",
        r"\\.\pipe\fixture",
        r"\\?\GLOBALROOT\Device\fixture",
        r"\\server\pipe\fixture",
    ] {
        let error = open_input(Path::new(value)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
    }
}
#[test]
fn directly_opened_file_symlink_is_rejected_after_handle_open() {
    let root = tempfile::tempdir().unwrap();
    let target = root.path().join("target");
    let link = root.path().join("link");
    std::fs::write(&target, b"value").unwrap();
    match std::os::windows::fs::symlink_file(&target, &link) {
        Ok(()) => {
            assert!(rejected_metadata(
                &std::fs::symlink_metadata(&link).unwrap()
            ));
            assert!(open_input(&link).is_err());
            assert!(crate::read_regular_file(&link, 5).is_err());
            eprintln!("FILE_SYMLINK_FIXTURE_EXECUTED");
        }
        Err(error) if error.raw_os_error() == Some(1314) => {
            eprintln!("FILE_SYMLINK_FIXTURE_UNAVAILABLE_PRIVILEGE_1314");
        }
        Err(error) => panic!("symlink fixture failed: {error}"),
    }
}

#[test]
fn lx_file_reparse_fixture_is_rejected_after_handle_open() {
    let Some(fixture) = std::env::var_os("BOUNDED_IO_REPARSE_FIXTURE") else {
        eprintln!("LX_FILE_REPARSE_FIXTURE_NOT_SUPPLIED");
        return;
    };
    let path = Path::new(&fixture);
    assert!(rejected_metadata(&std::fs::symlink_metadata(path).unwrap()));
    assert!(open_input(path).is_err());
    assert!(crate::read_regular_file(path, 7).is_err());
}
