use clap::{Arg, ArgAction, Command};

pub fn build_cli() -> Command {
    Command::new("Template CLI")
        .version("1.1")
        .about("Retrieve project templates from remote Git repository")
        .arg(
            Arg::new("repo")
                .short('r')
                .long("repo")
                .value_name("URL")
                .required(false)
                .help("Remote Git repository URL"),
        )
        .arg(
            Arg::new("branch")
                .short('b')
                .long("branch")
                .help("Repository branch to use"),
        )
        .arg(
            Arg::new("template")
                .help("Template name to use")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("target_dir")
                .short('d')
                .long("target-dir")
                .value_name("PATH")
                .help("Specify the target directory to copy templates to (default: ./)"),
        )
        .arg(
            Arg::new("original")
                .short('o')
                .long("original")
                .num_args(0..=1)
                .help("Fetch from the remote repository, with options to select"),
        )
        .arg(
            Arg::new("clear-cache")
                .short('c')
                .long("clear-cache")
                .action(ArgAction::SetTrue)
                .help("Clear the cache"),
        )
        .arg(
            Arg::new("check-cache")
                .short('x')
                .long("check-cache")
                .action(ArgAction::SetTrue)
                .help("Check and display cached configurations"),
        )
}