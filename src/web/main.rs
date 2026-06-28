//! `vmlab-web` — an Actix-web server that exposes vmlab over a REST + WebSocket
//! API and serves the embedded SolidJS console UI. It talks to the same
//! supervisor and lab daemons the CLI does, over the existing unix-socket
//! protocol; no daemon changes are involved.

mod api;
mod assets;
mod auth;
mod events;
mod state;
mod vnc;

use std::net::IpAddr;
use std::process::ExitCode;

use actix_web::middleware::from_fn;
use actix_web::{App, HttpServer, web};
use clap::Parser;

use state::{AppState, AuthConfig};

#[derive(Parser)]
#[command(name = "vmlab-web", version, about = "Web UI server for vmlab")]
struct Args {
    /// Address to bind (non-loopback implies --auth)
    #[arg(long, default_value = "127.0.0.1")]
    bind: IpAddr,
    /// TCP port
    #[arg(long, default_value_t = 7878)]
    port: u16,
    /// Require username/password login (auto-enabled for non-loopback binds)
    #[arg(long)]
    auth: bool,
    /// Login username (or VMLAB_WEB_USER)
    #[arg(long)]
    user: Option<String>,
    /// Login password, hashed once at startup (or VMLAB_WEB_PASSWORD; prefer a hash)
    #[arg(long)]
    password: Option<String>,
    /// Pre-computed argon2 PHC password hash (or VMLAB_WEB_PASSWORD_HASH)
    #[arg(long)]
    password_hash: Option<String>,
}

/// A CLI flag value, falling back to an environment variable.
fn or_env(flag: &Option<String>, var: &str) -> Option<String> {
    flag.clone().or_else(|| std::env::var(var).ok())
}

fn build_auth(args: &Args) -> Result<AuthConfig, String> {
    let enabled = args.auth || !args.bind.is_loopback();
    if !enabled {
        return Ok(AuthConfig {
            enabled: false,
            user: String::new(),
            password_hash: String::new(),
        });
    }
    let user = or_env(&args.user, "VMLAB_WEB_USER")
        .ok_or("auth is enabled but --user (or VMLAB_WEB_USER) is not set")?;
    let hash = or_env(&args.password_hash, "VMLAB_WEB_PASSWORD_HASH");
    let plain = or_env(&args.password, "VMLAB_WEB_PASSWORD");
    let password_hash = match (hash, plain) {
        (Some(h), _) => h,
        (None, Some(p)) => auth::hash_password(&p)?,
        (None, None) => {
            return Err(
                "auth is enabled but no --password / --password-hash (or env) is set".into(),
            );
        }
    };
    Ok(AuthConfig {
        enabled: true,
        user,
        password_hash,
    })
}

#[actix_web::main]
async fn main() -> ExitCode {
    let args = Args::parse();

    let auth = match build_auth(&args) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("vmlab-web: {e}");
            return ExitCode::FAILURE;
        }
    };

    // The lab in the working directory (if any) is the default; the switcher
    // also lists every lab the supervisor knows about.
    let default_lab = vmlab::cli::lab::current_lab().ok();
    match &default_lab {
        Some((name, root)) => println!("vmlab-web: default lab `{name}` ({})", root.display()),
        None => {
            println!("vmlab-web: no lab in the working directory (switcher lists running labs)")
        }
    }
    if auth.enabled {
        println!("vmlab-web: authentication enabled (user `{}`)", auth.user);
    }

    let data = web::Data::new(AppState::new(auth, default_lab));
    let (bind, port) = (args.bind, args.port);
    println!("vmlab-web: listening on http://{bind}:{port}");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(from_fn(auth::gate))
            // Auth (exempt from the gate).
            .route("/api/auth", web::get().to(auth::probe))
            .route("/api/login", web::post().to(auth::login))
            .route("/api/logout", web::post().to(auth::logout))
            // Labs.
            .route("/api/labs", web::get().to(api::list_labs))
            // VM sub-routes (literal before the `{action}` catch-all).
            .route(
                "/api/labs/{lab}/vms/{vm}/sendkeys",
                web::post().to(api::vm_sendkeys),
            )
            .route(
                "/api/labs/{lab}/vms/{vm}/screenshot.png",
                web::get().to(api::vm_screenshot),
            )
            .route(
                "/api/labs/{lab}/vms/{vm}/snapshots",
                web::get().to(api::vm_snapshots),
            )
            .route(
                "/api/labs/{lab}/vms/{vm}/snapshots/{name}",
                web::delete().to(api::snapshot_delete),
            )
            .route(
                "/api/labs/{lab}/vms/{vm}/{action}",
                web::post().to(api::vm_action),
            )
            // Snapshots (literal before the `{action}` catch-all).
            .route(
                "/api/labs/{lab}/snapshots/{name}/restore",
                web::post().to(api::snapshot_restore),
            )
            .route(
                "/api/labs/{lab}/snapshots",
                web::post().to(api::snapshot_take),
            )
            .route("/api/labs/{lab}/{action}", web::post().to(api::lab_action))
            .route("/api/labs/{lab}", web::get().to(api::lab_status))
            // Live streams.
            .route("/api/events", web::get().to(events::events))
            .route("/vnc/{lab}/{vm}", web::get().to(vnc::vnc))
            // SPA + static assets.
            .default_service(web::route().to(assets::spa))
    })
    // One worker keeps the cached daemon clients on a single runtime.
    .workers(1)
    .bind((bind, port));

    let server = match server {
        Ok(s) => s,
        Err(e) => {
            eprintln!("vmlab-web: cannot bind {bind}:{port}: {e}");
            return ExitCode::FAILURE;
        }
    };

    match server.run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("vmlab-web: {e}");
            ExitCode::FAILURE
        }
    }
}
