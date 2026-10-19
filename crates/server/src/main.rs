//! Kompozicijski korijen: jedino mjesto gdje se sve spaja (ARHITEKTURA §6.2).
//! Naredbe: `serve` (zadano), `migrate`, `openapi`.

use std::sync::Arc;

use anyhow::{Context, bail};
use server::config::Settings;
use server::shutdown;

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
/// Pravila v1 i pravila v2 (iz iste mape); oba se provjeravaju pri startu.
fn content_snapshots(
    dir: Option<&std::path::Path>,
) -> anyhow::Result<(content::Snapshot, content::Snapshot)> {
    let dir = dir.map_or_else(content::repo_dir, std::path::Path::to_path_buf);
    let read = |f: fn(&std::path::Path) -> Result<content::Snapshot, content::ContentError>| {
        let snap = f(&dir).map_err(|e| anyhow::anyhow!("sadržaj u {}: {e:?}", dir.display()))?;
        snap.build()
            .map_err(|e| anyhow::anyhow!("sadržaj u {} nije ispravan: {e:?}", dir.display()))?;
        anyhow::Ok(snap)
    };
    Ok((
        read(content::Snapshot::read)?,
        read(content::Snapshot::read_v2_checked)?,
    ))
}

/// AI iz okoline: bez ključa igra radi na rezervama (neutralne presude, kratki odgovori likova).
fn ai_port() -> Arc<dyn app::AiPort> {
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(k) if !k.trim().is_empty() => {
            let models = ai::Models::from_env();
            tracing::info!(presuda = %models.judge, lik = %models.character, mentor = %models.mentor, "AI uključen");
            Arc::new(ai::Anthropic::new(k, models))
        }
        _ => {
            tracing::warn!(
                "ANTHROPIC_API_KEY nije postavljen: AI je isključen, igra radi na rezervama"
            );
            Arc::new(app::DisabledAi)
        }
    }
}

fn deps(
    store: store::Store,
    snapshot: content::Snapshot,
    snapshot_v2: content::Snapshot,
) -> app::Deps {
    let s = Arc::new(store);
    app::Deps {
        status: s.clone(),
        log: s.clone(),
        guests: s.clone(),
        jobs: s,
        ai: ai_port(),
        ids: Arc::new(app::RandomIds),
        snapshot: Arc::new(snapshot),
        snapshot_v2: Arc::new(snapshot_v2),
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
/// zakup istekne i posao se preuzme ponovno (ARHITEKTURA §9.4). Gasi se signalom `stop`: posao u radu se
/// dovrši, pa petlja završi (`serve` čeka s rokom, tek zatim `abort()`).
fn spawn_worker(
    deps: app::Deps,
    mut stop: tokio::sync::watch::Receiver<bool>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while !*stop.borrow() {
            let d = deps.clone();
            let mut gasenje = stop.clone();
            let job = async move {
                app::run_next_job_until(&d, async move {
                    // gašenje: posao u radu dobiva JOB_GRACE da završi, pa se vraća u red
                    let _ = gasenje.wait_for(|v| *v).await;
                    tokio::time::sleep(JOB_GRACE).await;
                })
                .await
            };
            let pause = match tick(job).await {
                Tick::Worked => continue,
                Tick::Idle => std::time::Duration::from_millis(250),
                Tick::Failed => std::time::Duration::from_secs(1),
            };
            tokio::select! {
                () = tokio::time::sleep(pause) => {}
                _ = stop.changed() => {}
            }
        }
    })
}

/// Koliko posao u radu smije dovršiti nakon signala za gašenje; zatim se vraća u red (`defer`) da ga nova
/// instanca preuzme odmah. Railway šalje SIGKILL nakon `RAILWAY_DEPLOYMENT_DRAINING_SECONDS` (zadano 0 do 3 s,
/// podesivo), pa je to najbolji pokušaj; u dijelu 9 postaviti varijablu na 30 s.
const JOB_GRACE: std::time::Duration = std::time::Duration::from_secs(8);
/// Koliko `serve` čeka da se radnik ugasi nakon `JOB_GRACE` (zapis u red) prije `abort()`.
const WORKER_EXIT: std::time::Duration = std::time::Duration::from_secs(5);

async fn serve() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();
    let settings = Settings::from_env().context("neispravne postavke okoline")?;
    let database_url = settings
        .database_url
        .as_deref()
        .context("nedostaje DATABASE_URL (vidi .env.example)")?;
    if let Some(web) = &settings.web_dir
        && !web.join("index.html").is_file()
    {
        bail!(
            "HOTELIXSIM_WEB: u {} nema index.html (je li web izgrađen?)",
            web.display()
        );
    }
    let (snapshot, snapshot_v2) = content_snapshots(settings.content_dir.as_deref())?;
    let ruleset = snapshot.fingerprint();
    let store = store::Store::connect_lazy(database_url)?;
    let deps = deps(store, snapshot, snapshot_v2);
    let (stop, stopped) = tokio::sync::watch::channel(false);
    let mut worker = spawn_worker(deps.clone(), stopped);
    let app = api::router_with_config(api::AppState { deps }, settings.router_config());
    let addr = &settings.addr;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .with_context(|| format!("ne mogu slušati na {addr} (je li port zauzet?)"))?;
    tracing::info!(
        %ruleset,
        production = settings.production,
        lozinka = settings.site_password.is_some(),
        web = settings.web_dir.is_some(),
        "HotelixSim API sluša na http://{addr}"
    );
    shutdown::serve_until(listener, app, async move {
        shutdown::signal().await;
        // radnik prestaje preuzimati poslove ODMAH, usporedo s pražnjenjem HTTP veza
        stop.send_replace(true);
        tracing::info!("signal za gašenje: završavam zahtjeve u tijeku");
    })
    .await?;
    if tokio::time::timeout(JOB_GRACE + WORKER_EXIT, &mut worker)
        .await
        .is_err()
    {
        tracing::warn!(
            "radnik se nije ugasio u roku: prekidam (posao u radu se preuzima nakon isteka zakupa)"
        );
        worker.abort();
    }
    tracing::info!("ugašeno");
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
