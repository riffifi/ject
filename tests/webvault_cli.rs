#[test]
fn webvault_fetches_and_verifies_a_binary_snapshot() {
    use std::io::{Read, Write};
    use std::process::Command;

    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/asset", listener.local_addr().unwrap());
    let body = [0u8, 255, 13, 10, 42];
    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(5)))
            .unwrap();
        let mut request = [0u8; 1024];
        let count = stream.read(&mut request).unwrap();
        assert!(request[..count].starts_with(b"GET /asset HTTP/1.1"));
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
            )
            .unwrap();
        stream.write_all(&body).unwrap();
    });

    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let directory = std::env::temp_dir().join(format!(
        "ject-webvault-test-{}-{unique}",
        std::process::id()
    ));
    std::fs::create_dir(&directory).unwrap();
    let snapshot = directory.join("snapshot");
    let package = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/webvault");
    let binary = env!("CARGO_BIN_EXE_ject");
    let fetch = Command::new(binary)
        .current_dir(&package)
        .arg("run")
        .arg("--")
        .arg("fetch")
        .arg(&url)
        .arg(&snapshot)
        .output()
        .unwrap();
    assert!(
        fetch.status.success(),
        "{}",
        String::from_utf8_lossy(&fetch.stderr)
    );
    server.join().unwrap();

    assert_eq!(std::fs::read(snapshot.join("body.bin")).unwrap(), body);
    let metadata: serde_json::Value =
        serde_json::from_slice(&std::fs::read(snapshot.join("meta.json")).unwrap()).unwrap();
    assert_eq!(metadata["url"], url);
    assert_eq!(metadata["status"], 200);
    assert_eq!(metadata["size"], 5);
    assert_eq!(metadata["content_type"], "application/octet-stream");

    let show = Command::new(binary)
        .current_dir(&package)
        .arg("run")
        .arg("--")
        .arg("show")
        .arg(&snapshot)
        .output()
        .unwrap();
    assert!(
        show.status.success(),
        "{}",
        String::from_utf8_lossy(&show.stderr)
    );
    assert!(String::from_utf8_lossy(&show.stdout).contains("5 bytes verified"));

    let duplicate = Command::new(binary)
        .current_dir(&package)
        .arg("run")
        .arg("--")
        .arg("fetch")
        .arg(&url)
        .arg(&snapshot)
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
    assert!(String::from_utf8_lossy(&duplicate.stderr).contains("destination already exists"));

    std::fs::write(snapshot.join("body.bin"), [1u8]).unwrap();
    let altered = Command::new(binary)
        .current_dir(&package)
        .arg("run")
        .arg("--")
        .arg("show")
        .arg(&snapshot)
        .output()
        .unwrap();
    assert!(!altered.status.success());
    assert!(String::from_utf8_lossy(&altered.stderr).contains("does not match"));
    std::fs::remove_dir_all(&directory).unwrap();
}
