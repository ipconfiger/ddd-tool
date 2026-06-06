use clap::{Parser, Subcommand};

mod context;
pub use context::DddContext;

pub mod trait_def;
pub mod registry;
pub use trait_def::{DddCommand, CommandResult};
pub use registry::CommandRegistry;

#[derive(Parser, Debug)]
#[command(name = "ddd-tool")]
#[command(version = "0.1.0")]
#[command(about = "DocDriven CLI - 文档驱动开发框架")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Init(InitCmd),
    Prepare(PrepareCmd),
    Exec(ExecCmd),
    Verify(VerifyCmd),
    Audit(AuditCmd),
    Confirm(ConfirmCmd),
    Archive(ArchiveCmd),
    Report(ReportCmd),
    Final(FinalCmd),
    Sync(SyncCmd),
    /// 扫描 phases 目录，生成 phases 数组
    Accept,
    /// setup: 在项目级别配置命令和技能
    Setup(SetupCmd),
}

#[derive(Parser, Debug)]
pub struct InitCmd {
    #[arg(help = "需求上下文文档路径")]
    pub context: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PrepareCmd;

#[derive(Parser, Debug)]
pub struct ExecCmd;

#[derive(Parser, Debug)]
pub struct VerifyCmd;

#[derive(Parser, Debug)]
pub struct AuditCmd;

#[derive(Parser, Debug)]
pub struct ConfirmCmd;

#[derive(Parser, Debug)]
pub struct ArchiveCmd;

#[derive(Parser, Debug)]
pub struct ReportCmd;

#[derive(Parser, Debug)]
pub struct SyncCmd;

#[derive(Parser, Debug)]
pub struct FinalCmd;

/// setup: 在项目级别配置 Claude 或 OpenCode 的命令和技能
#[derive(Parser, Debug)]
pub struct SetupCmd {
    #[arg(long, value_enum, help = "目标工具: claude 或 opencode")]
    pub tool: Tool,
}

#[derive(clap::ValueEnum, Debug, Clone)]
pub enum Tool {
    Claude,
    Opencode,
}

pub fn run() {
    let cli = Cli::parse();

    match cli.command {
        Some(cmd) => dispatch(cmd),
        None => {
            println!("请使用 --help 查看可用命令");
        }
    }
}

fn dispatch(cmd: Command) {
    let ctx = match DddContext::new() {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("错误: {}", e);
            return;
        }
    };

    let registry = CommandRegistry::new();

    match cmd {
        Command::Init(c) => {
            let args = c.context.unwrap_or_default();
            dispatch_command(&registry, "init", &ctx, &args);
        }
        Command::Prepare(_) => dispatch_command(&registry, "prepare", &ctx, ""),
        Command::Exec(_) => dispatch_command(&registry, "exec", &ctx, ""),
        Command::Verify(_) => dispatch_command(&registry, "verify", &ctx, ""),
        Command::Audit(_) => dispatch_command(&registry, "audit", &ctx, ""),
        Command::Confirm(_) => dispatch_command(&registry, "confirm", &ctx, ""),
        Command::Archive(_) => dispatch_command(&registry, "archive", &ctx, ""),
        Command::Report(_) => dispatch_command(&registry, "report", &ctx, ""),
        Command::Final(_) => dispatch_command(&registry, "final", &ctx, ""),
        Command::Sync(_) => dispatch_command(&registry, "sync", &ctx, ""),
        Command::Accept => {
            let _ = internal::accept();
        }
        Command::Setup(c) => setup::run(c, &registry),
    }
}

fn dispatch_command(registry: &CommandRegistry, name: &str, ctx: &DddContext, args: &str) {
    match registry.get(name) {
        Some(cmd) => {
            if !cmd.is_cli_visible() {
                eprintln!("错误: 命令 '{}' 不可直接调用", name);
                return;
            }
            match cmd.execute(ctx, args) {
                Ok(result) => {
                    if result.success {
                        if let Some(ref prompt) = result.prompt {
                            println!("{}", prompt);
                        }
                        if !result.message.is_empty() && result.prompt.is_none() {
                            println!("{}", result.message);
                        }
                    } else {
                        eprintln!("错误: {}", result.message);
                    }
                }
                Err(e) => {
                    eprintln!("错误: {}", e);
                }
            }
        }
        None => {
            eprintln!("未知命令: {}", name);
        }
    }
}

pub mod init;
pub mod prepare;
pub mod exec;
pub mod verify;
pub mod audit;
mod confirm_phase;
pub mod archive;
pub mod report;
pub mod sync;
pub mod internal;
pub mod setup;
mod final_verify;
