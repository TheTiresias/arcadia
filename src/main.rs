mod build;
mod config;
mod content;
mod frontmatter;
mod markdown;
mod new;
mod serve;
mod templates;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "arcadia", about = "Static site generator")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold a new site (or a post / deck / fiction story)
    New {
        #[command(subcommand)]
        kind: Option<NewKind>,
    },

    /// Build the site
    Build {
        /// Content directory
        #[arg(long)]
        src: Option<String>,

        /// Output directory
        #[arg(long)]
        output: Option<String>,

        /// Include draft posts
        #[arg(long)]
        drafts: bool,
    },

    /// Build and serve the site locally
    Serve {
        /// Content directory
        #[arg(long)]
        src: Option<String>,

        /// Output directory
        #[arg(long)]
        output: Option<String>,

        /// Port to listen on
        #[arg(long)]
        port: Option<u16>,

        /// Include draft posts
        #[arg(long)]
        drafts: bool,
    },

    /// Copy embedded templates into embed/ for local customisation
    Eject,

    /// Delete the output directory
    Clean {
        /// Output directory to delete
        #[arg(long)]
        output: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
enum NewKind {
    /// Create a new post
    Post { slug: String },

    /// Create a new slide deck
    Deck { slug: String },

    /// Create a new fiction story
    Fiction { slug: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli).await {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Command::New { kind } => match kind {
            None => new::scaffold_site(&PathBuf::from("."))?,
            Some(NewKind::Post { slug }) => new::new_post(&PathBuf::from("."), &slug)?,
            Some(NewKind::Deck { slug }) => new::new_deck(&PathBuf::from("."), &slug)?,
            Some(NewKind::Fiction { slug }) => {
                new::new_fiction(&PathBuf::from("."), &slug)?
            }
        },
        Command::Eject => new::eject_templates(&PathBuf::from("."))?,
        Command::Build { src, output, drafts } => {
            let site_config = config::SiteConfig::load(std::path::Path::new("."))?;
            let src_dir = src
                .or_else(|| site_config.content_dir.clone())
                .unwrap_or_else(|| "example".to_owned());
            let out_dir = output
                .or_else(|| site_config.output_dir.clone())
                .unwrap_or_else(|| "dist".to_owned());
            let config = build::BuildConfig::load(
                PathBuf::from("."),
                PathBuf::from(src_dir),
                PathBuf::from(out_dir),
                drafts,
                &site_config,
            );
            build::build(&config)?;
        }
        Command::Serve { src, output, port, drafts } => {
            let site_config = config::SiteConfig::load(std::path::Path::new("."))?;
            let src_dir = src
                .or_else(|| site_config.content_dir.clone())
                .unwrap_or_else(|| "example".to_owned());
            let out_dir = output
                .or_else(|| site_config.output_dir.clone())
                .unwrap_or_else(|| "dist".to_owned());
            let resolved_port = port.or(site_config.port).unwrap_or(3000);
            let config = build::BuildConfig::load(
                PathBuf::from("."),
                PathBuf::from(src_dir),
                PathBuf::from(out_dir),
                drafts,
                &site_config,
            );
            serve::serve(&config, resolved_port).await?;
        }
        Command::Clean { output, yes } => {
            let site_config = config::SiteConfig::load(std::path::Path::new("."))?;
            let out_dir = output
                .or_else(|| site_config.output_dir.clone())
                .unwrap_or_else(|| "dist".to_owned());
            let path = PathBuf::from(&out_dir);

            if !path.exists() {
                println!("{} does not exist, nothing to clean.", out_dir);
                return Ok(());
            }

            if !yes {
                use std::io::{self, Write};
                print!("Delete {}? [y/N]: ", out_dir);
                io::stdout().flush()?;
                let mut input = String::new();
                io::stdin().read_line(&mut input)?;
                if !matches!(input.trim().to_lowercase().as_str(), "y" | "yes") {
                    println!("Aborted.");
                    return Ok(());
                }
            }

            std::fs::remove_dir_all(&path)
                .with_context(|| format!("delete {}", out_dir))?;
            println!("Deleted {}.", out_dir);
        }
    }
    Ok(())
}
