// SPDX-FileCopyrightText: 2026 Open Health Technology Foundation contributors
// SPDX-License-Identifier: AGPL-3.0-or-later

#![cfg(all(unix, feature = "cli"))]

//! `opengrowth serve` stops cleanly on SIGTERM, which `docker stop`, systemd,
//! and Kubernetes send, as well as on Ctrl-C (SIGINT).

use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("a free port")
        .local_addr()
        .expect("an address")
        .port()
}

/// Starts the server and waits until it accepts connections.
fn start() -> Child {
    let port = free_port();
    let child = Command::new(env!("CARGO_BIN_EXE_opengrowth"))
        .args(["serve", "--bind", &format!("127.0.0.1:{port}")])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the server starts");
    let deadline = Instant::now() + Duration::from_secs(10);
    while TcpStream::connect(("127.0.0.1", port)).is_err() {
        assert!(
            Instant::now() < deadline,
            "the server did not start listening"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    child
}

/// Sends `signal` and waits up to 5 s for the server to exit.
fn stop_with(signal: &str) -> std::process::ExitStatus {
    let mut child = start();
    let status = Command::new("kill")
        .args([&format!("-{signal}"), &child.id().to_string()])
        .status()
        .expect("kill runs");
    assert!(status.success());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(exit) = child.try_wait().expect("the server's status") {
            return exit;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("the server did not stop within 5 s of SIG{signal}");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn sigterm_stops_the_server_cleanly() {
    assert!(stop_with("TERM").success());
}

#[test]
fn sigint_stops_the_server_cleanly() {
    assert!(stop_with("INT").success());
}
