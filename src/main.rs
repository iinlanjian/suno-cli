mod api;
mod auth;
mod cli;
mod config;
mod download;
mod errors;
mod output;

use clap::Parser;

use api::SunoClient;
use api::types::GenerateRequest;
use auth::AuthState;
use cli::*;
use errors::CliError;
use output::OutputFormat;

fn client() -> Result<SunoClient, CliError> {
    let auth = AuthState::load()?;
    SunoClient::new(auth)
}

fn build_tags(tags: Option<&str>, vocal: Option<&VocalGender>) -> Option<String> {
    let mut parts: Vec<&str> = Vec::new();
    if let Some(t) = tags {
        parts.push(t);
    }
    match vocal {
        Some(VocalGender::Male) => parts.push("male vocals"),
        Some(VocalGender::Female) => parts.push("female vocals"),
        None => {}
    }
    if parts.is_empty() {
        None
    } else {
        Some(parts.join(", "))
    }
}

/// Generate, wait, optionally download — shared by generate/describe/extend.
async fn handle_generation(
    c: &SunoClient,
    clips: Vec<api::types::Clip>,
    wait: bool,
    download_dir: Option<&str>,
    fmt: &OutputFormat,
    quiet: bool,
) -> Result<(), CliError> {
    let ids: Vec<String> = clips.iter().map(|c| c.id.clone()).collect();

    if wait && !ids.is_empty() {
        if !quiet {
            eprintln!("Waiting for generation to complete...");
        }
        let final_clips = c.poll_clips(&ids, 600).await?;

        if let Some(dir) = download_dir {
            for clip in &final_clips {
                if clip.status == "complete" {
                    let path = download::download_clip(clip, dir, false).await?;
                    if !quiet {
                        eprintln!("Downloaded: {path}");
                    }
                }
            }
        }

        match fmt {
            OutputFormat::Json => output::json::success(&final_clips),
            OutputFormat::Table => output::table::clips(&final_clips),
        }
    } else {
        match fmt {
            OutputFormat::Json => output::json::success(&clips),
            OutputFormat::Table => {
                output::table::clips(&clips);
                if !ids.is_empty() {
                    eprintln!("\nUse `suno status {}` to check progress", ids.join(" "));
                }
            }
        }
    }
    Ok(())
}

async fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let fmt = OutputFormat::detect(cli.json);

    match cli.command {
        Commands::Auth(args) => {
            let mut state = AuthState::load().unwrap_or_default();
            if let Some(jwt) = args.jwt {
                state.jwt = Some(jwt);
            }
            if let Some(cookie) = args.cookie {
                state.cookie = Some(cookie);
            }
            if let Some(session) = args.session {
                state.session_id = Some(session);
            }
            if let Some(device) = args.device {
                state.device_id = Some(device);
            }
            state.save()?;

            let client = SunoClient::new(state)?;
            let info = client.billing_info().await?;
            eprintln!(
                "Authenticated! Plan: {}, Credits: {}",
                info.plan.name, info.total_credits_left
            );
        }

        Commands::Credits => {
            let info = client()?.billing_info().await?;
            match fmt {
                OutputFormat::Json => output::json::success(&info),
                OutputFormat::Table => output::table::billing(&info),
            }
        }

        Commands::Models => {
            let info = client()?.billing_info().await?;
            match fmt {
                OutputFormat::Json => output::json::success(&info.models),
                OutputFormat::Table => output::table::models(&info.models),
            }
        }

        Commands::List(args) => {
            let feed = client()?.feed(args.page).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&feed.clips),
                OutputFormat::Table => output::table::clips(&feed.clips),
            }
        }

        Commands::Search(args) => {
            // Search across all pages for matching clips
            let c = client()?;
            let mut found = Vec::new();
            let mut page = 0u32;
            let query_lower = args.query.to_lowercase();
            loop {
                let feed = c.feed(page).await?;
                for clip in &feed.clips {
                    let title_match = clip.title.to_lowercase().contains(&query_lower);
                    let tag_match = clip
                        .metadata
                        .tags
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&query_lower);
                    if title_match || tag_match {
                        found.push(clip.clone());
                    }
                }
                if !feed.has_more || page > 10 {
                    break;
                }
                page += 1;
            }
            match fmt {
                OutputFormat::Json => output::json::success(&found),
                OutputFormat::Table => {
                    if found.is_empty() {
                        eprintln!("No clips matching \"{}\"", args.query);
                    } else {
                        output::table::clips(&found);
                    }
                }
            }
        }

        Commands::Lyrics(args) => {
            if !cli.quiet {
                eprintln!("Generating lyrics...");
            }
            let result = client()?.generate_lyrics(&args.prompt).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&result),
                OutputFormat::Table => output::table::lyrics(&result),
            }
        }

        Commands::Generate(args) => {
            let lyrics = match (&args.lyrics, &args.lyrics_file) {
                (Some(l), _) => Some(l.clone()),
                (_, Some(path)) => Some(std::fs::read_to_string(path)?),
                _ => None,
            };
            let tags = build_tags(args.tags.as_deref(), args.vocal.as_ref());
            let req = GenerateRequest {
                mv: args.model.to_api_key().to_string(),
                prompt: lyrics,
                gpt_description_prompt: None,
                title: args.title,
                tags,
                negative_tags: args.exclude,
                make_instrumental: args.instrumental,
                generation_type: Some("TEXT".into()),
                token: args.token,
                continue_clip_id: None,
                continue_at: None,
                task: None,
                weirdness: args.weirdness,
                style_influence: args.style_influence,
                variation_category: args.variation.map(|v| v.to_api_value().to_string()),
            };

            if !cli.quiet {
                eprintln!("Submitting generation ({})...", args.model.display_name());
            }
            let c = client()?;
            let clips = c.generate(&req).await?;
            handle_generation(
                &c,
                clips,
                args.wait,
                args.download.as_deref(),
                &fmt,
                cli.quiet,
            )
            .await?;
        }

        Commands::Describe(args) => {
            let tags = build_tags(args.tags.as_deref(), args.vocal.as_ref());
            let req = GenerateRequest {
                mv: args.model.to_api_key().to_string(),
                prompt: Some(String::new()),
                gpt_description_prompt: Some(args.prompt),
                title: None,
                tags,
                negative_tags: None,
                make_instrumental: args.instrumental,
                generation_type: Some("TEXT".into()),
                token: None,
                continue_clip_id: None,
                continue_at: None,
                task: None,
                weirdness: args.weirdness,
                style_influence: args.style_influence,
                variation_category: None,
            };

            if !cli.quiet {
                eprintln!("Submitting description ({})...", args.model.display_name());
            }
            let c = client()?;
            let clips = c.generate(&req).await?;
            handle_generation(
                &c,
                clips,
                args.wait,
                args.download.as_deref(),
                &fmt,
                cli.quiet,
            )
            .await?;
        }

        Commands::Extend(args) => {
            let req = GenerateRequest {
                mv: "chirp-fenix".into(),
                prompt: args.lyrics,
                gpt_description_prompt: None,
                title: None,
                tags: args.tags,
                negative_tags: None,
                make_instrumental: false,
                generation_type: Some("TEXT".into()),
                token: None,
                continue_clip_id: Some(args.clip_id),
                continue_at: Some(args.at),
                task: None,
                weirdness: None,
                style_influence: None,
                variation_category: None,
            };

            let c = client()?;
            let clips = c.generate(&req).await?;
            handle_generation(&c, clips, args.wait, None, &fmt, cli.quiet).await?;
        }

        Commands::Concat(args) => {
            let clip = client()?.concat(&args.clip_id).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&clip),
                OutputFormat::Table => output::table::clips(&[clip]),
            }
        }

        Commands::Cover(args) => {
            let clip = client()?.cover(&args.clip_id, args.tags.as_deref()).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&clip),
                OutputFormat::Table => output::table::clips(&[clip]),
            }
        }

        Commands::Remaster(args) => {
            let clip = client()?
                .remaster(&args.clip_id, args.model.to_api_key())
                .await?;
            match fmt {
                OutputFormat::Json => output::json::success(&clip),
                OutputFormat::Table => output::table::clips(&[clip]),
            }
        }

        Commands::Stems(args) => {
            let clip = client()?.stems(&args.clip_id).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&clip),
                OutputFormat::Table => output::table::clips(&[clip]),
            }
        }

        Commands::Status(args) => {
            let clips = client()?.get_clips(&args.ids).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&clips),
                OutputFormat::Table => output::table::clips(&clips),
            }
        }

        Commands::Download(args) => {
            let c = client()?;
            let clips = c.get_clips(&args.ids).await?;
            if clips.is_empty() {
                return Err(CliError::NotFound(format!(
                    "clips: {}",
                    args.ids.join(", ")
                )));
            }
            let mut paths = Vec::new();
            for clip in &clips {
                let path = download::download_clip(clip, &args.output, args.video).await?;
                if !cli.quiet {
                    eprintln!("Downloaded: {path}");
                }
                paths.push(path);
            }
            match fmt {
                OutputFormat::Json => output::json::success(&paths),
                OutputFormat::Table => {}
            }
        }

        Commands::Delete(args) => {
            if args.ids.is_empty() {
                return Err(CliError::Config("no clip IDs provided".into()));
            }
            if !args.yes {
                eprintln!(
                    "Deleting {} clip(s): {}",
                    args.ids.len(),
                    args.ids.join(", ")
                );
                eprintln!("Use -y to skip confirmation, or press Ctrl+C to cancel");
                // Give 2 seconds to cancel
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            client()?.delete_clips(&args.ids).await?;
            eprintln!("Deleted {} clip(s)", args.ids.len());
        }

        Commands::Config(args) => match args.action {
            ConfigAction::Show => {
                let cfg = config::AppConfig::load();
                println!("{}", serde_json::to_string_pretty(&cfg)?);
            }
            ConfigAction::Set { key, value } => {
                eprintln!("config set {key}={value} — not yet implemented (use env vars SUNO_*)");
            }
            ConfigAction::Check => {
                let _ = config::AppConfig::load();
                match AuthState::load() {
                    Ok(auth) => {
                        if auth.is_jwt_expired() {
                            eprintln!("Auth: JWT expired — run `suno auth`");
                        } else {
                            eprintln!("Auth: OK");
                        }
                    }
                    Err(_) => eprintln!("Auth: not configured — run `suno auth`"),
                }
            }
        },

        Commands::AgentInfo => {
            let info = serde_json::json!({
                "name": "suno",
                "version": env!("CARGO_PKG_VERSION"),
                "commands": [
                    "generate", "describe", "lyrics", "extend", "concat",
                    "cover", "remaster", "stems", "list", "search", "status",
                    "download", "delete", "credits", "models", "auth", "config", "agent-info"
                ],
                "models": {
                    "v5.5": "chirp-fenix",
                    "v5": "chirp-crow",
                    "v4.5+": "chirp-bluejay",
                    "v4.5": "chirp-auk",
                    "v4": "chirp-v4",
                },
                "features": [
                    "tags", "negative_tags", "vocal_gender", "weirdness",
                    "style_influence", "variation_category", "instrumental",
                    "extend", "concat", "cover", "remaster", "stems", "lyrics",
                    "search", "delete"
                ],
                "env_prefix": "SUNO_",
                "auth_required": true,
                "default_model": "chirp-fenix (v5.5)",
            });
            println!("{}", serde_json::to_string_pretty(&info)?);
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        let json_mode = std::env::args().any(|a| a == "--json")
            || !std::io::IsTerminal::is_terminal(&std::io::stdout());

        if json_mode {
            output::json::error(e.error_code(), &e.to_string());
        } else {
            eprintln!("Error: {e}");
        }
        std::process::exit(e.exit_code());
    }
}
