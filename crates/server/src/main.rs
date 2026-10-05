//! Kompozicijski korijen: jedino mjesto gdje se sve spaja (ARHITEKTURA §6.2).
//! Naredbe: `serve` (zadano), `migrate`, `openapi`.

use std::sync::Arc;

use anyhow::{Context, bail};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "serve".to_owned());
    match cmd.as_str() {
        "openapi" => {
            print!("{}", api::openapi_json());
            Ok(())
        }
        "migrate" => {
            let store = store::Store::connect_lazy(&database_url()?)?;
            store.migrate().await.context("migracija nije uspjela")?;
            println!("migracije primijenjene");
            Ok(())
        }
        "serve" => serve().await,
        other => bail!("nepoznata naredba: {other} (serve | migrate | openapi)"),
    }
}

fn database_url() -> anyhow::Result<String> {
    std::env::var("DATABASE_URL").context("nedostaje DATABASE_URL (vidi .env.example)")
}

/// Sadržaj se učitava i provjerava pri startu: server s pokvarenim katalogom ne smije ni krenuti (ADR-015 t. 12).
fn content_snapshot() -> anyhow::Result<content::Snapshot> {
    let dir = std::env::var("HOTELIXSIM_CONTENT")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|_| content::repo_dir());
    let snap = content::Snapshot::read(&dir)
        .map_err(|e| anyhow::anyhow!("sadržaj u {}: {e:?}", dir.display()))?;
    snap.build()
        .map_err(|e| anyhow::anyhow!("sadržaj u {} nije ispravan: {e:?}", dir.display()))?;
    Ok(snap)
}

fn deps(store: store::Store, snapshot: content::Snapshot) -> app::Deps {
    let s = Arc::new(store);
    app::Deps {
        status: s.clone(),
        log: s.clone(),
        guests: s.clone(),
        jobs: s,
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(snapshot),
    }
}

async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let addr = std::env::var("HOTELIXSIM_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".to_owned());
    let snapshot = content_snapshot()?;
    let ruleset = snapshot.fingerprint();
    let store = store::Store::connect_lazy(&database_url()?)?;
    let app = api::router(api::AppState {
        deps: deps(store, snapshot),
    });
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("ne mogu slušati na {addr} (je li port zauzet?)"))?;
    tracing::info!(%ruleset, "HotelixSim API sluša na http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    Ok(())
}
