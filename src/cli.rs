use clap::Parser;

use crate::cloner::{clone, BackupGitlabOptions, CloneParams, FetchGitlabOptions, FilterPatterns};
use anyhow::{bail, Result};

#[derive(Parser)]
#[clap(author, version, about)]
/// A tool for cloning all available repositories in a GitLab instance
struct Cli {
    /// The GitLab instance URL for fetch repositories (example: https://gitlab.local/)
    #[clap(long, value_parser, value_name = "FETCH URL")]
    fu: String,

    /// Your personal GitLab token for fetch repositories
    #[clap(long, value_parser, value_name = "FETCH TOKEN")]
    ft: String,

    #[clap(long, value_parser, value_name = "BACKUP URL")]
    /// The GitLab instance URL for backup repositories (example: https://backup-gitlab.local/)
    bu: Option<String>,

    #[clap(long, value_parser, value_name = "BACKUP TOKEN")]
    /// Your personal GitLab token for backup repositories
    bt: Option<String>,

    #[clap(long, value_parser, value_name = "BACKUP GROUP")]
    /// A target created group on backup GitLab for push repositories
    bg: Option<String>,

    #[clap(
        long,
        multiple_values = true,
        value_delimiter = ',',
        value_name = "PATTERNS"
    )]
    /// Comma separated include regexp patterns (cannot be used together with --exclude flag)
    include: Option<Vec<String>>,

    #[clap(
        long,
        multiple_values = true,
        value_delimiter = ',',
        value_name = "PATTERNS"
    )]
    /// Comma separated exclude regexp patterns (cannot be used together with --include flag)
    exclude: Option<Vec<String>>,

    /// A destination local folder for save downloaded repositories
    #[clap(long, short, value_parser, value_name = "DIRECTORY")]
    dst: Option<String>,

    /// Verbose level (one or more, max four)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(long)]
    /// Show all projects to download
    dry_run: bool,

    #[clap(long, value_parser, value_name = "COUNT")]
    /// Low-level option, how many projects can fetch in one request
    objects_per_page: Option<u32>,

    #[clap(long, value_parser, value_name = "COUNT")]
    /// Maximum projects to download
    limit: Option<usize>,

    #[clap(long, value_parser, default_value_t = 21, value_name = "LIMIT")]
    /// Limit concurrency download
    concurrency_limit: usize,

    #[clap(long)]
    /// Download projects explicitly owned by user
    only_owned: bool,

    #[clap(long)]
    /// Download only user's projects
    only_membership: bool,

    /// Enable download by ssh instead of http. An authorized ssh key is required
    #[clap(long)]
    download_ssh: bool,

    /// Enable upload by ssh instead of http. An authorized ssh key is required
    #[clap(long)]
    upload_ssh: bool,

    /// Disable saving the directory hierarchy
    #[clap(long)]
    disable_hierarchy: bool,
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();

    let log_level = match cli.verbose {
        0 => tracing::Level::ERROR,
        1 => tracing::Level::WARN,
        2 => tracing::Level::INFO,
        3 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    let fetch_gl = FetchGitlabOptions::new(cli.fu, cli.ft)?;

    let patterns = if cli.exclude.is_some() && cli.include.is_some() {
        bail!("You cannot use the --include and --exclude flag together");
    } else if let Some(patterns) = cli.exclude {
        Some(FilterPatterns::Exclude(patterns))
    } else {
        cli.include.map(FilterPatterns::Include)
    };

    let backup_gl = if let (Some(url), Some(token), Some(group)) = (cli.bu, cli.bt, cli.bg) {
        Some(BackupGitlabOptions::new(url, token, group)?)
    } else {
        None
    };

    let clone_params = CloneParams {
        fetch: fetch_gl,
        dst: cli.dst,
        backup: backup_gl,
        patterns,
        dry_run: cli.dry_run,
        objects_per_page: cli.objects_per_page,
        limit: cli.limit,
        concurrency_limit: cli.concurrency_limit,
        only_owned: cli.only_owned,
        only_membership: cli.only_membership,
        download_ssh: cli.download_ssh,
        upload_ssh: cli.upload_ssh,
        disable_hierarchy: cli.disable_hierarchy,
    };

    clone(clone_params)
}
