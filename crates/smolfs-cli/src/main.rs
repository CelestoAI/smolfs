use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use smolfs_core::{InitVolume, MountVolume, SmolFsHome};
use smolfs_juicefs::{SmolFs, doctor, install_managed_juicefs};

#[derive(Debug, Parser)]
#[command(
    name = "smolfs",
    version,
    about = "Durable developer volumes for agents",
    long_about = "SmolFS manages JuiceFS-backed agent workspaces from one CLI. Use `smolfs doctor` first, then `smolfs init NAME --dev` for a local test volume."
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(about = "Check JuiceFS, FUSE, and local SmolFS setup")]
    Doctor {
        #[arg(
            long,
            help = "Copy a discovered JuiceFS binary into SmolFS' managed bin directory"
        )]
        install: bool,
        #[arg(long, help = "Print a machine-readable setup report")]
        json: bool,
    },
    #[command(about = "Create a named SmolFS volume")]
    Init {
        #[arg(help = "Volume name, using letters, numbers, '.', '_' or '-'")]
        name: String,
        #[arg(long, help = "Create a local JuiceFS volume for development")]
        dev: bool,
        #[arg(long, help = "JuiceFS metadata URL, such as redis://localhost:6379/1")]
        metadata: Option<String>,
        #[arg(
            long,
            help = "Object store URL, such as s3://bucket/prefix or file:///tmp/objects"
        )]
        store: Option<String>,
        #[arg(
            long,
            help = "JuiceFS storage type escape hatch, such as s3, gs, or file"
        )]
        storage: Option<String>,
        #[arg(long, help = "JuiceFS bucket/endpoint used with --storage")]
        bucket: Option<String>,
    },
    #[command(about = "Mount a SmolFS volume at a local path")]
    Mount {
        #[arg(help = "Existing SmolFS volume name")]
        name: String,
        #[arg(help = "Local directory where the volume should be mounted")]
        path: PathBuf,
        #[arg(
            long,
            help = "Run JuiceFS in the foreground instead of background mode"
        )]
        foreground: bool,
        #[arg(
            long,
            help = "Ask JuiceFS to test object storage access before mounting"
        )]
        check_storage: bool,
    },
    #[command(about = "Show configured SmolFS volumes")]
    Status {
        #[arg(help = "Optional volume name to inspect")]
        name: Option<String>,
        #[arg(long, help = "Print machine-readable status")]
        json: bool,
    },
    #[command(about = "Best-effort flush check for a mounted volume")]
    Flush {
        #[arg(help = "Mounted SmolFS volume name")]
        name: String,
    },
    #[command(about = "Unmount a SmolFS volume and wait for JuiceFS flush")]
    Unmount {
        #[arg(help = "Mounted SmolFS volume name")]
        name: String,
        #[arg(long, help = "Force unmount a busy mountpoint")]
        force: bool,
    },
    #[command(about = "Alias for `smolfs unmount`")]
    Umount {
        #[arg(help = "Mounted SmolFS volume name")]
        name: String,
        #[arg(long, help = "Force unmount a busy mountpoint")]
        force: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Doctor { install, json } => {
            let home = SmolFsHome::from_env()?;
            if install {
                let path = install_managed_juicefs(&home)?;
                println!("Installed managed JuiceFS binary at {}", path.display());
            }
            let report = doctor(&home)?;
            if json {
                println!("{}", serde_json::to_string_pretty(&report)?);
            } else {
                println!("SmolFS home: {}", report.home.display());
                println!("Config: {}", report.config.display());
                if report.juicefs.found {
                    println!(
                        "JuiceFS: {}{}",
                        report
                            .juicefs
                            .path
                            .as_ref()
                            .map(|path| path.display().to_string())
                            .unwrap_or_else(|| "(unknown)".into()),
                        if report.juicefs.managed {
                            " (managed)"
                        } else {
                            ""
                        }
                    );
                    if let Some(version) = report.juicefs.version {
                        println!("Version: {version}");
                    }
                } else {
                    println!("JuiceFS: missing");
                    println!("Fix: run `smolfs doctor --install` or set SMOLFS_JUICEFS_BIN");
                }

                if report.fuse.found {
                    println!("FUSE: {}", report.fuse.detail);
                } else {
                    println!("FUSE: missing ({})", report.fuse.detail);
                    if let Some(fix) = report.fuse.fix {
                        println!("Fix: {fix}");
                    }
                }
            }
        }
        Command::Init {
            name,
            dev,
            metadata,
            store,
            storage,
            bucket,
        } => {
            let fs = SmolFs::from_env()?;
            let volume = fs.init(InitVolume {
                name,
                dev,
                metadata_url: metadata,
                store_url: store,
                storage,
                bucket,
            })?;
            println!(
                "Initialized volume {} ({})",
                volume.name,
                if volume.dev { "dev" } else { "cloud" }
            );
        }
        Command::Mount {
            name,
            path,
            foreground,
            check_storage,
        } => {
            let fs = SmolFs::from_env()?;
            let mount = fs.mount(MountVolume {
                name,
                path,
                foreground,
                check_storage,
            })?;
            println!(
                "Mounted volume {} at {}",
                mount.name,
                mount.mountpoint.display()
            );
        }
        Command::Status { name, json } => {
            let fs = SmolFs::from_env()?;
            let status = fs.status(name.as_deref())?;
            if json {
                println!("{}", serde_json::to_string_pretty(&status)?);
            } else if status.volumes.is_empty() {
                println!("No volumes");
            } else {
                for volume in status.volumes {
                    let mountpoint = volume
                        .mountpoint
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| "-".into());
                    println!(
                        "{}\t{}\t{}\t{}",
                        volume.name,
                        if volume.dev { "dev" } else { "cloud" },
                        volume.storage,
                        mountpoint
                    );
                }
            }
        }
        Command::Flush { name } => {
            let fs = SmolFs::from_env()?;
            fs.flush(&name)?;
            println!("Flushed volume {name}");
        }
        Command::Unmount { name, force } | Command::Umount { name, force } => {
            let fs = SmolFs::from_env()?;
            fs.unmount(&name, force)?;
            println!("Unmounted volume {name}");
        }
    }

    Ok(())
}
