//! Uredno gašenje (pilot, dio 2 c). Pravi SIGTERM/Ctrl-C se ne šalje procesu testa: testira se funkcija koja
//! prima futureove za gašenje. Da `shutdown::signal()` stvarno sluša SIGTERM (Railway) provjerava ručno dio 9.
use std::future::pending;
use std::time::Duration;

use server::shutdown::{first_of, serve_until};
use tokio::time::timeout;

#[tokio::test]
async fn first_of_finishes_when_either_signal_arrives() {
    timeout(Duration::from_secs(2), first_of(async {}, pending::<()>()))
        .await
        .expect("prvi signal");
    timeout(Duration::from_secs(2), first_of(pending::<()>(), async {}))
        .await
        .expect("drugi signal (SIGTERM uz Ctrl-C)");
}

#[tokio::test]
async fn first_of_waits_while_neither_signal_arrived() {
    let r = timeout(
        Duration::from_millis(150),
        first_of(pending::<()>(), pending::<()>()),
    )
    .await;
    assert!(r.is_err(), "bez signala gašenja nema");
}

/// Mali HTTP/1.0 zahtjev običnim utičnicama (bez dodatnih značajki tokio-a).
fn http_get(addr: std::net::SocketAddr, path: &str) -> String {
    use std::io::{Read, Write};
    let mut s = std::net::TcpStream::connect(addr).unwrap();
    s.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
    write!(s, "GET {path} HTTP/1.0\r\nHost: test\r\n\r\n").unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    out
}

#[tokio::test]
async fn serve_until_serves_then_stops_when_shutdown_future_completes() {
    let (deps, _mem) = app::memory::deps(content::Snapshot::read(&content::repo_dir()).unwrap());
    let router = api::router(api::AppState { deps });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
    let server = tokio::spawn(serve_until(listener, router, async move {
        let _ = stopped.await;
    }));

    let res = tokio::task::spawn_blocking(move || http_get(addr, "/v1/health"))
        .await
        .unwrap();
    assert!(
        res.starts_with("HTTP/1.0 200") || res.starts_with("HTTP/1.1 200"),
        "{res}"
    );

    stop.send(()).unwrap();
    timeout(Duration::from_secs(5), server)
        .await
        .expect("poslužitelj se ugasio nakon signala")
        .unwrap()
        .expect("uredno gašenje bez greške");
}
