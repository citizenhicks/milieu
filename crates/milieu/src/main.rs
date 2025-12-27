mod api;
mod auth;
mod commands;
mod config;
mod crypto;
mod keys;
mod error;
mod keychain;
mod manifest;
mod repo;
mod style;

use clap::{Parser, Subcommand};
use error::Result;

#[derive(Parser, Debug)]
#[command(
    name = "milieu",
    version,
    about = "rust e2ee dotenv sync",
    after_help = "tip: run `milieu <command> --help` for examples."
)]
struct Cli {
    #[arg(long, default_value = "default")]
    profile: String,

    #[arg(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "create an account for milieu", after_help = "example: milieu register")]
    Register,
    #[command(about = "login to your milieu account", after_help = "example: milieu login")]
    Login,
    #[command(about = "remove local auth and UMK from your keychain", after_help = "example: milieu logout")]
    Logout,
    #[command(
        about = "in your project directory, initialize the repo .milieu",
        after_help = "examples:\n  milieu init\n  milieu init --name my_repo"
    )]
    Init {
        #[arg(long)]
        name: Option<String>,
    },
    #[command(
        about = "clone a repo into this folder",
        after_help = "examples:\n  milieu clone --repo my-app\n  milieu clone"
    )]
    Clone {
        #[arg(long)]
        repo: Option<String>,
    },
    #[command(
        about = "add a dotenv file to the current branch",
        after_help = "examples:\n  milieu add .env\n  milieu add .env.local --tag dev\n  milieu add .env --branch prod"
    )]
    Add {
        path: String,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(
        about = "remove a dotenv file from the current branch",
        after_help = "examples:\n  milieu remove .env\n  milieu remove .env.local --branch prod"
    )]
    Remove {
        path: String,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(
        about = "show version history for a file",
        after_help = "example: milieu log .env"
    )]
    Log {
        path: String,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(
        about = "checkout a specific version of a file",
        after_help = "example: milieu checkout .env --version 3"
    )]
    Checkout {
        path: String,
        #[arg(long)]
        version: u32,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(
        about = "show diffs for a file or all files",
        after_help = "examples:\n  milieu changes\n  milieu changes .env\n  milieu changes .env --version 3\n  milieu changes --branch prod"
    )]
    Changes {
        path: Option<String>,
        #[arg(long)]
        version: Option<u32>,
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(
        about = "list repos linked to your user",
        after_help = "example: milieu repos list"
    )]
    Repos {
        #[command(subcommand)]
        command: ReposCommand,
    },
    #[command(
        name = "branch",
        about = "manage repo branches and their dotenv files",
        after_help = "examples:\n  milieu branch list\n  milieu branch add dev --file .env\n  milieu branch set dev"
    )]
    Branch {
        #[command(subcommand)]
        command: BranchCommand,
    },
    #[command(about = "push branch changes to the server", after_help = "example: milieu push --branch dev")]
    Push {
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(about = "download and decrypt dotenv files for a branch", after_help = "example: milieu pull --branch dev")]
    Pull {
        #[arg(long)]
        branch: Option<String>,
    },
    #[command(about = "show local vs remote state for this repo", after_help = "example: milieu status")]
    Status {
        #[arg(long)]
        json: bool,
    },
    #[command(about = "check system prerequisites and configuration", after_help = "example: milieu doctor")]
    Doctor,
    #[command(
        about = "manage recovery phrase",
        after_help = "examples:\n  milieu phrase show\n  milieu phrase status"
    )]
    Phrase {
        #[command(subcommand)]
        command: PhraseCommand,
    },
    #[command(about = "list active sessions for this user", after_help = "example: milieu sessions")]
    Sessions,
}

#[derive(Subcommand, Debug)]
enum BranchCommand {
    #[command(about = "list branches in this repo", after_help = "example: milieu branch list")]
    List,
    #[command(
        about = "add a branch with one or more dotenv files",
        after_help = "examples:\n  milieu branch add dev --file .env\n  milieu branch add dev --file .env.local --tag dev"
    )]
    Add {
        name: String,
        #[arg(long, action = clap::ArgAction::Append)]
        file: Vec<String>,
        #[arg(long, action = clap::ArgAction::Append)]
        tag: Vec<String>,
    },
    #[command(
        about = "remove a branch from the manifest",
        after_help = "example: milieu branch remove prod"
    )]
    Remove { name: String },
    #[command(
        name = "set",
        about = "set the default branch",
        after_help = "example: milieu branch set dev"
    )]
    Set { name: String },
}

#[derive(Subcommand, Debug)]
enum ReposCommand {
    #[command(about = "list repos linked to this user", after_help = "example: milieu repos list")]
    List,
    #[command(
        about = "manage repo sharing and invites",
        after_help = "examples:\n  milieu repos manage list --repo my-app\n  milieu repos manage add --repo my-app --email someone@acme.com --access write\n  milieu repos manage invites\n  milieu repos manage share --repo my-app\n  milieu repos manage delete --repo my-app"
    )]
    Manage {
        #[command(subcommand)]
        command: ManageCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ManageCommand {
    #[command(about = "list repo collaborators and pending invites", after_help = "example: milieu repos manage list --repo my-app")]
    List {
        #[arg(long)]
        repo: String,
    },
    #[command(about = "invite a collaborator by email", after_help = "example: milieu repos manage add --repo my-app --email a@b.com --access write")]
    Add {
        #[arg(long)]
        repo: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        access: String,
    },
    #[command(about = "update a collaborator or invite access", after_help = "example: milieu repos manage set --repo my-app --email a@b.com --access read")]
    Set {
        #[arg(long)]
        repo: String,
        #[arg(long)]
        email: String,
        #[arg(long)]
        access: String,
    },
    #[command(about = "remove a collaborator or revoke invite", after_help = "example: milieu repos manage remove --repo my-app --email a@b.com")]
    Remove {
        #[arg(long)]
        repo: String,
        #[arg(long)]
        email: String,
    },
    #[command(about = "list and respond to pending invites", after_help = "example: milieu repos manage invites")]
    Invites,
    #[command(about = "accept an invite by id", after_help = "example: milieu repos manage accept inv_123")]
    Accept { invite_id: String },
    #[command(about = "reject an invite by id", after_help = "example: milieu repos manage reject inv_123")]
    Reject { invite_id: String },
    #[command(
        about = "share repo key with active collaborators",
        after_help = "example: milieu repos manage share --repo my-app"
    )]
    Share {
        #[arg(long)]
        repo: String,
    },
    #[command(
        about = "delete a remote repo and all ciphertext",
        after_help = "example: milieu repos manage delete --repo my-app"
    )]
    Delete {
        #[arg(long)]
        repo: String,
    },
}

#[derive(Subcommand, Debug)]
enum PhraseCommand {
    #[command(about = "show the recovery phrase from keychain")]
    Show,
    #[command(about = "check if recovery phrase exists in keychain")]
    Status,
}

#[tokio::main]
async fn main() {
    if let Err(err) = run().await {
        let message = format!("Error: {}", err);
        eprintln!("{}", style::paint(style::RED, &message));
        std::process::exit(1);
    }
}

async fn run() -> Result<()> {
    if std::env::args().len() == 1 {
        print_banner_and_help()?;
        return Ok(());
    }

    let cli = Cli::parse();
    init_tracing(cli.verbose);

    match cli.command {
        Commands::Register => commands::register::run(&cli.profile).await?,
        Commands::Login => commands::login::run(&cli.profile).await?,
        Commands::Logout => commands::logout::run(&cli.profile).await?,
        Commands::Init { name } => commands::init::run(&cli.profile, name).await?,
        Commands::Clone { repo } => commands::clone::run(&cli.profile, repo).await?,
        Commands::Repos { command } => match command {
            ReposCommand::List => commands::repos::list(&cli.profile).await?,
            ReposCommand::Manage { command } => match command {
                ManageCommand::List { repo } => {
                    commands::repos::manage_list(&cli.profile, &repo).await?
                }
                ManageCommand::Add { repo, email, access } => {
                    commands::repos::manage_add(&cli.profile, &repo, &email, &access).await?
                }
                ManageCommand::Set { repo, email, access } => {
                    commands::repos::manage_set(&cli.profile, &repo, &email, &access).await?
                }
                ManageCommand::Remove { repo, email } => {
                    commands::repos::manage_remove(&cli.profile, &repo, &email).await?
                }
                ManageCommand::Invites => commands::repos::manage_invites(&cli.profile).await?,
                ManageCommand::Accept { invite_id } => {
                    commands::repos::manage_accept(&cli.profile, &invite_id).await?
                }
                ManageCommand::Reject { invite_id } => {
                    commands::repos::manage_reject(&cli.profile, &invite_id).await?
                }
                ManageCommand::Share { repo } => {
                    commands::repos::manage_share(&cli.profile, &repo).await?
                }
                ManageCommand::Delete { repo } => {
                    commands::repos::manage_delete(&cli.profile, &repo).await?
                }
            },
        },
        Commands::Sessions => commands::sessions::list(&cli.profile).await?,
        Commands::Branch { command } => match command {
            BranchCommand::List => commands::branches::list()?,
            BranchCommand::Add { name, file, tag } => {
                commands::branches::add_and_sync(&cli.profile, &name, file, tag).await?
            }
            BranchCommand::Remove { name } => {
                commands::branches::remove_and_sync(&cli.profile, &name).await?
            }
            BranchCommand::Set { name } => {
                commands::branches::set_default_and_sync(&cli.profile, &name).await?
            }
        },
        Commands::Add { path, tag, branch } => {
            commands::add::run(&path, tag, branch)?
        }
        Commands::Remove { path, branch } => {
            commands::remove::run(&path, branch)?
        }
        Commands::Log { path, branch } => {
            commands::log::run(&cli.profile, path, branch).await?
        }
        Commands::Checkout {
            path,
            version,
            branch,
        } => {
            commands::checkout::run(&cli.profile, path, version, branch).await?
        }
        Commands::Changes { path, branch, version } => {
            commands::changes::run(&cli.profile, path, branch, version).await?
        }
        Commands::Push { branch } => commands::push::run(&cli.profile, branch).await?,
        Commands::Pull { branch } => commands::pull::run(&cli.profile, branch).await?,
        Commands::Status { json } => commands::status::run(&cli.profile, json).await?,
        Commands::Doctor => commands::doctor::run(&cli.profile)?,
        Commands::Phrase { command } => match command {
            PhraseCommand::Show => commands::phrase::show(&cli.profile)?,
            PhraseCommand::Status => commands::phrase::status(&cli.profile)?,
        },
    }

    Ok(())
}

fn init_tracing(verbosity: u8) {
    let level = match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    };

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

fn print_banner_and_help() -> Result<()> {
    use clap::CommandFactory;
    println!("{}", style::bold(style::MAUVE, "MILIEU"));

    let status = login_status();
    println!("{}", style::paint(style::LAVENDER, &status));
    println!();

    let mut cmd = Cli::command();
    println!("{}", cmd.render_usage());
    println!();

    println!("{}", style::bold(style::MAUVE, "User commands:"));
    print_grouped_commands(
        &cmd,
        &["register", "login", "logout", "doctor", "phrase", "repos", "sessions"],
    );
    println!();

    println!("{}", style::bold(style::MAUVE, "Repo commands:"));
    print_grouped_commands(&cmd, &["init", "clone", "status", "branch"]);
    println!();

    println!("{}", style::bold(style::MAUVE, "Branch commands:"));
    print_grouped_commands(
        &cmd,
        &["add", "remove", "push", "pull", "changes", "log", "checkout"],
    );
    println!();
    Ok(())
}

fn print_grouped_commands(cmd: &clap::Command, names: &[&str]) {
    let width = names.iter().map(|name| name.len()).max().unwrap_or(0);
    for name in names {
        let desc = lookup_about(cmd, name).unwrap_or_default();
        let line = if desc.is_empty() {
            format!("  {}", name)
        } else {
            format!("  {:<width$}  {}", name, desc, width = width)
        };
        println!("{}", style::paint(style::TEXT, &line));
    }
}

fn lookup_about(cmd: &clap::Command, name: &str) -> Option<String> {
    cmd.get_subcommands()
        .find(|sub| sub.get_name() == name)
        .and_then(|sub| sub.get_about().or_else(|| sub.get_long_about()))
        .map(|value| value.to_string())
}

fn login_status() -> String {
    match auth::load_user_id("default") {
        Ok(user_id) => format!("status: logged in ({})", user_id),
        Err(_) => "status: not logged in".to_string(),
    }
}
