//! Where the breach check's bytes are allowed to go — R-25, and the half `hibp.rs`'s own tests
//! could not honestly make.
//!
//! **This is a separate test binary on purpose.** The first claim below needs `ALL_PROXY` and
//! `HTTPS_PROXY` set in the process, and environment variables are process-global while `cargo
//! test` runs a binary's tests on threads. In `hibp.rs` that would have raced every other test
//! that builds a client. Here the binary holds exactly these two, and they take [`SERIAL`] so the
//! window in which a proxy variable is set is a window no other client is constructed in — a
//! separate binary bounds the blast radius to this file, and the lock is what actually closes it.
//!
//! # The finding that produced this file
//!
//! `hibp.rs` first carried a unit test asserting `client.agent.config().proxy().is_none()`. It
//! passed with the `.proxy(None)` override **deleted**, because `Proxy::try_from_env` returns
//! `None` when the environment has no proxy in it — the test measured this machine's shell rather
//! than this project's code. An egress rule is only tested by watching the place the bytes would
//! otherwise have gone, which is what these two do.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::{Mutex, mpsc};
use std::thread;
use std::time::Duration;

use trustvault_lib::hibp::RangeClient;

/// A listener that records the first request it is given and answers a minimal range.
///
/// Returns its port and a channel that stays empty for as long as nothing connects — which is the
/// assertion both tests below actually make.
fn listening() -> (u16, mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("a free port");
    let port = listener.local_addr().expect("bound").port();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else {
                return;
            };
            let mut buffer = [0_u8; 2048];
            let read = stream.read(&mut buffer).unwrap_or(0);
            if tx
                .send(String::from_utf8_lossy(&buffer[..read]).into_owned())
                .is_err()
            {
                return;
            }
            let body = "0000000000000000000000000000000000A:1\r\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    (port, rx)
}

/// Long enough that a connection which was going to arrive has arrived.
const SETTLE: Duration = Duration::from_millis(250);

/// Serializes the two tests in this binary.
///
/// `cargo test` runs a binary's tests on threads, and one of them sets a process-global proxy
/// variable. Without this, the other could build its client inside that window and assert against
/// a configuration the test it belongs to never chose.
static SERIAL: Mutex<()> = Mutex::new(());

/// A proxy in the environment does not get to see the prefix.
///
/// The failure mode this guards is not exotic. A user on a corporate laptop, or anyone who has
/// ever exported `ALL_PROXY` for a shell, would have had every range request routed through a host
/// TrustVault never named — and R-25's packet capture, taken on a developer's machine with no
/// proxy set, would have recorded a perfectly clean run.
#[test]
fn a_proxy_in_the_environment_never_sees_a_request() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (proxy_port, proxy_saw) = listening();
    let (target_port, target_saw) = listening();

    // SAFETY: this binary's tests are serialized (see the module docs) and nothing else in the
    // process reads a proxy variable. Setting it before the client is built is required, because
    // ureq reads the environment when the config is constructed rather than per request.
    unsafe {
        std::env::set_var("ALL_PROXY", format!("http://127.0.0.1:{proxy_port}"));
        std::env::set_var("HTTPS_PROXY", format!("http://127.0.0.1:{proxy_port}"));
    }

    let client = RangeClient::new(&format!("http://127.0.0.1:{target_port}"));
    let answered = client.range("5BAA6");

    unsafe {
        std::env::remove_var("ALL_PROXY");
        std::env::remove_var("HTTPS_PROXY");
    }

    assert!(answered.is_ok(), "the request went somewhere: {answered:?}");
    let asked = target_saw
        .recv_timeout(SETTLE)
        .expect("the endpoint we named is the one that answered");
    assert!(asked.starts_with("GET /range/5BAA6 "), "{asked}");
    assert!(
        proxy_saw.recv_timeout(SETTLE).is_err(),
        "the prefix went through a proxy the user's environment chose"
    );
}

/// A plain-HTTP endpoint is refused before anything is sent.
///
/// `RangeClient` turns `https_only` off for a loopback address so its own tests can use a plain
/// listener, and this is the assertion that the exemption is that narrow: the same listener,
/// addressed as `localhost` rather than `127.0.0.1`, is never contacted at all.
#[test]
fn a_plain_http_endpoint_is_refused_rather_than_downgraded_to() {
    let _serial = SERIAL
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let (port, saw) = listening();

    let client = RangeClient::new(&format!("http://localhost:{port}"));
    let refused = client.range("5BAA6");

    assert!(refused.is_err(), "plain HTTP is not a transport for this");
    assert!(
        saw.recv_timeout(SETTLE).is_err(),
        "the request was sent over plaintext before being refused"
    );
}
