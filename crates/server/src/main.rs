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
        ai: Arc::new(app::DisabledAi),
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(snapshot),
    }
}

/// Ishod jedne iteracije radnika.
#[derive(Debug, PartialEq, Eq)]
enum Tick {
    /// Posao je obrađen: odmah traži sljedeći.
    Worked,
    /// Red je prazan.
    Idle,
    /// Posao je javio grešku ili je zadatak paničario (zakup će istjecati pa se posao preuzme ponovno).
    Failed,
}

/// Jedna iteracija u VLASTITOM tokio zadatku: panika u poslu ne ubija radnika, nego se zabilježi
/// i petlja nastavlja. Posao u radu ostaje `running`, zakup istekne i posao se preuzme (ARHITEKTURA §9.4);
/// deterministička panika nakon 5 pokušaja postaje `failed`.
async fn tick<Fut, E>(job: Fut) -> Tick
where
    Fut: std::future::Future<Output = Result<bool, E>> + Send + 'static,
    E: std::fmt::Display + Send + 'static,
{
    match tokio::spawn(job).await {
        Ok(Ok(true)) => Tick::Worked,
        Ok(Ok(false)) => Tick::Idle,
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "posao reda nije uspio");
            Tick::Failed
        }
        Err(e) if e.is_panic() => {
            tracing::error!(error = %e, "radnik: panika u poslu reda; nastavljam, posao se preuzima nakon isteka zakupa");
            Tick::Failed
        }
        Err(e) => {
            tracing::error!(error = %e, "radnik: zadatak prekinut");
            Tick::Failed
        }
    }
}

/// Radnik reda poslova u istom procesu (demo; zaseban način `work` je za pilot). Pad procesa ne gubi posao:
/// zakup istekne i posao se preuzme ponovno (ARHITEKTURA §9.4). Gasi se `abort()`-om iz `serve`.
fn spawn_worker(deps: app::Deps) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let d = deps.clone();
            match tick(async move { app::run_next_job(&d).await }).await {
                Tick::Worked => {}
                Tick::Idle => tokio::time::sleep(std::time::Duration::from_millis(250)).await,
                Tick::Failed => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
            }
        }
    })
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
    let deps = deps(store, snapshot);
    let worker = spawn_worker(deps.clone());
    let app = api::router(api::AppState { deps });
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .with_context(|| format!("ne mogu slušati na {addr} (je li port zauzet?)"))?;
    tracing::info!(%ruleset, "HotelixSim API sluša na http://{addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;
    worker.abort();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn panic_in_job_is_survived_and_reported_as_failed() {
        let t = tick(async {
            if true {
                panic!("namjerna panika u testu");
            }
            Ok::<bool, String>(true)
        })
        .await;
        assert_eq!(t, Tick::Failed);
    }

    #[tokio::test]
    async fn job_results_map_to_ticks() {
        assert_eq!(tick(async { Ok::<_, String>(true) }).await, Tick::Worked);
        assert_eq!(tick(async { Ok::<_, String>(false) }).await, Tick::Idle);
        assert_eq!(
            tick(async { Err::<bool, _>("x".to_owned()) }).await,
            Tick::Failed
        );
    }
}
