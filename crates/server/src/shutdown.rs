//! Uredno gašenje (pilot, dio 2 c): Ctrl-C ili SIGTERM (Railway, `docker stop`). Na Windowsu samo Ctrl-C.

use std::future::Future;

/// Završava čim završi prvi od dva futurea.
pub async fn first_of<A, B>(a: A, b: B)
where
    A: Future<Output = ()>,
    B: Future<Output = ()>,
{
    tokio::select! {
        () = a => {}
        () = b => {}
    }
}

/// Završava na Ctrl-C ili (na Unixu) SIGTERM.
pub async fn signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut s) => {
                s.recv().await;
            }
            Err(e) => {
                tracing::warn!(error = %e, "ne mogu slušati SIGTERM");
                std::future::pending::<()>().await;
            }
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    first_of(ctrl_c, terminate).await;
}

/// Poslužuje dok `shutdown` ne završi, pa dovršava zahtjeve u tijeku.
pub async fn serve_until(
    listener: tokio::net::TcpListener,
    app: axum::Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}
