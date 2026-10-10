use std::io::{self, Read};
struct InterruptedOnce(bool);
impl Read for InterruptedOnce {
    fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
        if !self.0 {
            self.0 = true;
            return Err(io::ErrorKind::Interrupted.into());
        }
        b"value".as_slice().read(output)
    }
}
#[test]
fn caller_interruption_policy_is_explicit() {
    assert_eq!(
        zixcel_bounded_io::read_bounded(InterruptedOnce(false), 5)
            .unwrap_err()
            .kind(),
        io::ErrorKind::Interrupted
    );
    // Finite cursor after one Interrupted: compare standard read_to_end behavior.
    struct Finite {
        interrupted: bool,
        data: io::Cursor<&'static [u8]>,
    }
    impl Read for Finite {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            if !self.interrupted {
                self.interrupted = true;
                return Err(io::ErrorKind::Interrupted.into());
            }
            self.data.read(output)
        }
    }
    let fixture = || Finite {
        interrupted: false,
        data: io::Cursor::new(b"value"),
    };
    let mut expected = Vec::new();
    fixture().read_to_end(&mut expected).unwrap();
    assert_eq!(
        zixcel_bounded_io::read_bounded_retrying(fixture(), 5).unwrap(),
        expected
    );
    let error = zixcel_bounded_io::read_bounded_retrying(fixture(), 4).unwrap_err();
    let limit = error
        .get_ref()
        .unwrap()
        .downcast_ref::<zixcel_bounded_io::InputLimit>()
        .unwrap();
    assert_eq!((limit.limit, limit.observed_at_least), (4, 5));
}
