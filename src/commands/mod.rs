use clap::{Parser, Subcommand};

mod context;
pub use context::DddContext;

#[derive(Parser, Debug)]
#[command(name = "ddd")]
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
    FixPlan(FixPlanCmd),
    FixExec(FixExecCmd),
    Archive(ArchiveCmd),
    Report(ReportCmd),
    Sync(SyncCmd),
    Resume(ResumeCmd),
    /// 扫描 phrases 目录，生成 phrases 数组
    GenPhrase,
    /// 设置当前 phase 状态为 issue_found
    SetIssue,
    /// 完成修复
    FinishFix,
    /// 完成阶段
    FinishPhrase,
}

#[derive(Parser, Debug)]
pub struct InitCmd {
    #[arg(long, help = "需求上下文文档路径")]
    pub context: Option<String>,
}

#[derive(Parser, Debug)]
pub struct PrepareCmd;

#[derive(Parser, Debug)]
pub struct ExecCmd;

#[derive(Parser, Debug)]
pub struct VerifyCmd;

#[derive(Parser, Debug)]
pub struct FixPlanCmd;

#[derive(Parser, Debug)]
pub struct FixExecCmd;

#[derive(Parser, Debug)]
pub struct ArchiveCmd;

#[derive(Parser, Debug)]
pub struct ReportCmd;

#[derive(Parser, Debug)]
pub struct SyncCmd;

#[derive(Parser, Debug)]
pub struct ResumeCmd;

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
    match cmd {
        Command::Init(c) => init::run(c),
        Command::Prepare(c) => prepare::run(c),
        Command::Exec(c) => exec::run(c),
        Command::Verify(c) => verify::run(c),
        Command::FixPlan(c) => fix_plan::run(c),
        Command::FixExec(c) => fix_exec::run(c),
        Command::Archive(c) => archive::run(c),
        Command::Report(c) => report::run(c),
        Command::Sync(c) => sync::run(c),
        Command::Resume(c) => resume::run(c),
        Command::GenPhrase => { let _ = internal::gen_phrase(); },
        Command::SetIssue => { let _ = internal::set_issue(); },
        Command::FinishFix => { let _ = internal::finish_fix(); },
        Command::FinishPhrase => { let _ = internal::finish_phrase(); },
    }
}

pub mod init;
pub mod prepare;
pub mod exec;
pub mod verify;
pub mod fix_plan;
pub mod fix_exec;
pub mod archive;
pub mod report;
pub mod sync;
pub mod resume;
pub mod internal;
