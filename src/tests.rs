use super::*;
use std::io::Cursor;
use std::thread;
use std::time::Duration;

#[test]
fn test_connection() {
    const TEST_DATA: &[u8; 5] = b"12345";

    thread::spawn(move|| {
        // Start up the server
        Networked::new(Cursor::new(&TEST_DATA[..]), ("127.0.0.1", 4000))
            .unwrap()
            .listen()
            .unwrap();
    });

    // Wait for the server to start up.
    thread::sleep(Duration::from_millis(500));

    // Connect to it
    let mut reader = NetworkReader::new(("127.0.0.1", 4000)).unwrap();

    let mut buf = [0u8; 4];
    reader.seek(SeekFrom::Start(1)).unwrap();
    reader.read_exact(&mut buf).unwrap();
    assert_eq!(&buf[..], &TEST_DATA[1..]);
}
