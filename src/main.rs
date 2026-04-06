mod api;
mod auth;
mod cli;
mod config;
mod download;
mod errors;
mod output;

use clap::Parser;

use api::SunoClient;
use api::types::{ControlSliders, GenerateMetadata, GenerateRequest, SetMetadataRequest};
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

/// Build metadata with control sliders (weirdness, style influence).
fn build_metadata(
    weirdness: Option<f64>,
    style_influence: Option<f64>,
) -> Option<GenerateMetadata> {
    if weirdness.is_none() && style_influence.is_none() {
        return None;
    }
    Some(GenerateMetadata {
        control_sliders: Some(ControlSliders {
            // Normalize 0-100 → 0.0-1.0
            weirdness_constraint: weirdness.map(|w| (w / 100.0).clamp(0.0, 1.0)),
            style_weight: style_influence.map(|s| (s / 100.0).clamp(0.0, 1.0)),
        }),
    })
}

/// Generate, wait, optionally download with lyrics embedding.
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

                    // Embed lyrics into MP3
                    let plain_lyrics = clip.metadata.prompt.as_deref();
                    // Try to get timed lyrics for synced display
                    let aligned = c.aligned_lyrics(&clip.id).await.ok();
                    download::embed_lyrics_in_mp3(
                        &path,
                        &clip.title,
                        plain_lyrics,
                        aligned.as_deref(),
                    )?;

                    if !quiet {
                        eprintln!("Downloaded: {path} (lyrics embedded)");
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
            let feed = client()?.search(&args.query).await?;
            match fmt {
                OutputFormat::Json => output::json::success(&feed.clips),
                OutputFormat::Table => {
                    if feed.clips.is_empty() {
                        eprintln!("No clips matching \"{}\"", args.query);
                    } else {
                        output::table::clips(&feed.clips);
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
            let metadata = build_metadata(args.weirdness, args.style_influence);

            let c = client()?;

            // Check captcha before generating
            if let Ok(captcha_needed) = c.check_captcha().await {
                if captcha_needed {
                    if args.token.is_none() {
                        eprintln!(
                            "Warning: captcha required. Use --token <hcaptcha_token> or solve captcha in browser."
                        );
                        eprintln!(
                            "Tip: Premier accounts with 200+ credits consumed usually skip captcha."
                        );
                    }
                }
            }

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
                metadata,
            };

            if !cli.quiet {
                eprintln!("Submitting generation ({})...", args.model.display_name());
            }
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
            let metadata = build_metadata(args.weirdness, args.style_influence);

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
                metadata,
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
                metadata: None,
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

                // Embed lyrics into MP3 downloads
                if !args.video {
                    let plain_lyrics = clip.metadata.prompt.as_deref();
                    let aligned = c.aligned_lyrics(&clip.id).await.ok();
                    download::embed_lyrics_in_mp3(
                        &path,
                        &clip.title,
                        plain_lyrics,
                        aligned.as_deref(),
                    )?;
                    if !cli.quiet {
                        eprintln!("Embedded lyrics into {path}");
                    }
                }

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
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            client()?.delete_clips(&args.ids).await?;
            eprintln!("Deleted {} clip(s)", args.ids.len());
        }

        Commands::Set(args) => {
            let lyrics = match (&args.lyrics, &args.lyrics_file) {
                (Some(l), _) => Some(l.clone()),
                (_, Some(path)) => Some(std::fs::read_to_string(path)?),
                _ => None,
            };
            let req = SetMetadataRequest {
                title: args.title.clone(),
                lyrics,
                caption: args.caption.clone(),
                remove_image_cover: if args.remove_cover { Some(true) } else { None },
                remove_video_cover: None,
            };
            client()?.set_metadata(&args.id, &req).await?;
            let mut changes = Vec::new();
            if args.title.is_some() {
                changes.push("title");
            }
            if args.lyrics.is_some() || args.lyrics_file.is_some() {
                changes.push("lyrics");
            }
            if args.caption.is_some() {
                changes.push("caption");
            }
            if args.remove_cover {
                changes.push("cover removed");
            }
            eprintln!("Updated: {}", changes.join(", "));
        }

        Commands::Publish(args) => {
            let c = client()?;
            let is_public = !args.private;
            for id in &args.ids {
                c.set_visibility(id, is_public).await?;
            }
            let state = if is_public { "public" } else { "private" };
            eprintln!("Set {} clip(s) to {state}", args.ids.len());
        }

        Commands::TimedLyrics(args) => {
            let words = client()?.aligned_lyrics(&args.id).await?;
            if args.lrc {
                // LRC format: [mm:ss.xx] word
                for w in &words {
                    if !w.success {
                        continue;
                    }
                    let mins = (w.start_s / 60.0) as u32;
                    let secs = w.start_s % 60.0;
                    println!("[{:02}:{:05.2}] {}", mins, secs, w.word);
                }
            } else {
                match fmt {
                    OutputFormat::Json => output::json::success(&words),
                    OutputFormat::Table => {
                        for w in &words {
                            if w.success {
                                println!("{:>6.2}s - {:>6.2}s  {}", w.start_s, w.end_s, w.word);
                            }
                        }
                    }
                }
            }
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
                    "download", "delete", "set", "publish", "timed-lyrics",
                    "credits", "models", "auth", "config", "agent-info"
                ],
                "models": {
                    "v5.5": "chirp-fenix",
                    "v5": "chirp-crow",
                    "v4.5+": "chirp-bluejay",
                    "v4.5": "chirp-auk",
                    "v4": "chirp-v4",
                },
                "features": [
                    "tags", "negative_tags", "vocal_gender",
                    "weirdness (metadata.control_sliders.weirdness_constraint)",
                    "style_influence (metadata.control_sliders.style_weight)",
                    "instrumental", "extend", "concat", "cover", "remaster",
                    "stems", "lyrics", "timed_lyrics", "set_metadata",
                    "set_visibility", "search", "delete", "captcha_check",
                    "id3_lyrics_embedding"
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
