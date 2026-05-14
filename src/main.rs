mod batch;
mod cli;
mod config;
mod logger;
mod tech;
mod util;
mod watcher;

mod context;

use std::{
    process,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

use crate::{
    batch::{base::Batch, data::BatchData},
    cli::base::Cli,
    config::{
        base::Config,
        batch::{data::BatchData as BatchDataCfg, ovr_opts::apply_first_batch_overrides},
        tech::Tech,
    },
    context::ContextData,
    logger::{base::Logger, level::LogLevel},
    util::get_ifname_from_src_ip,
};

use crate::tech::ext::TechExt;

#[tokio::main]
async fn main() {
    // Parse CLI arguments.
    let cli = Cli::parse();

    // Load configuration from file.
    let mut cfg = Config::load_from_file(&cli.args.config).unwrap_or_else(|e| {
        println!("[WARNING] Failed to load configuration file: {}", e);

        Config::default()
    });

    // Initialize the logger.
    let logger_cfg = cfg.logger.clone();

    let logger = Logger::new(
        logger_cfg.level.unwrap_or_default(),
        logger_cfg.path,
        logger_cfg.path_is_file,
        logger_cfg.date_format_file,
        logger_cfg.date_format_line,
        cli.args.watch,
    );

    logger
        .log_msg(LogLevel::Trace, "Logger initialized...")
        .ok();

    // Create the batch.
    logger
        .log_msg(LogLevel::Trace, "Initializing batch...")
        .ok();

    // Get override options.
    let ovr_opts = match cfg.batch.ovr_opts.clone() {
        Some(ovr_opts_cfg) => match ovr_opts_cfg.try_into() {
            Ok(ovr_opts) => Some(ovr_opts),
            Err(e) => {
                logger
                    .log_msg(
                        LogLevel::Fatal,
                        &format!("Failed to convert batch override options: {}", e),
                    )
                    .ok();

                process::exit(1);
            }
        },
        None => None,
    };

    let mut batch = Batch::new(
        cfg.batch.batches.iter().map(|b| b.clone().into()).collect(),
        ovr_opts,
    );

    // Check for first batch override.
    {
        let mut first_batch = {
            if let Some(first_batch) = batch.batches.first() {
                first_batch.clone()
            } else {
                BatchData::default()
            }
        };

        match apply_first_batch_overrides(&mut first_batch, &cli.args) {
            Ok(overriden) => {
                if overriden {
                    logger
                        .log_msg(
                            LogLevel::Info,
                            "Applied first batch overrides from CLI arguments...",
                        )
                        .ok();

                    if batch.batches.is_empty() {
                        batch.batches.push(first_batch.clone());
                    } else {
                        batch.batches[0] = first_batch.clone();
                    }

                    // Override the first batch in the config in the case for listing below.
                    let cfg_batch_cnt = cfg.batch.batches.len();

                    if cfg_batch_cnt > 0 {
                        cfg.batch.batches[0] = first_batch.clone().into()
                    } else {
                        cfg.batch
                            .batches
                            .push(BatchDataCfg::from(first_batch.clone()));
                    }
                } else {
                    logger
                        .log_msg(
                            LogLevel::Debug,
                            "No first batch overrides applied from CLI arguments.",
                        )
                        .ok();
                }
            }
            Err(e) => {
                logger
                    .log_msg(
                        LogLevel::Fatal,
                        &format!(
                            "Failed to apply first batch overrides from CLI arguments: {}",
                            e
                        ),
                    )
                    .ok();

                process::exit(1);
            }
        }
    }

    // Check if we should list and exit.
    if cli.args.list_cfg {
        cfg.list();

        process::exit(0);
    }

    // If we don't have any batches, there is an issue at this point.
    if batch.batches.is_empty() {
        logger
            .log_msg(
                LogLevel::Fatal,
                "No batches defined in configuration after applying overrides.",
            )
            .ok();

        process::exit(1);
    }

    // Create the tech.
    logger.log_msg(LogLevel::Trace, "Initializing tech...").ok();

    let tech = match cfg.tech.clone().try_into() {
        Ok(tech) => tech,
        Err(e) => {
            logger
                .log_msg(
                    LogLevel::Fatal,
                    &format!("Failed to initialize tech (conversion with config): {}", e),
                )
                .ok();

            process::exit(1);
        }
    };

    // Now we need to initialize the global context.
    logger
        .log_msg(LogLevel::Info, "Initializing context...")
        .ok();

    let ctx = ContextData::new(cfg, logger, cli, tech, batch);

    // Before getting to the tech and batches, let's try to retrieve a fallback interface.
    let iface_fb = {
        let batch_read = ctx.batch.read().await;
        let src_ip_opt = batch_read
            .batches
            .first()
            .and_then(|b| b.opt_ip.src.as_ref())
            .and_then(|src_vec| src_vec.first());

        if let Some(src_ip) = src_ip_opt {
            let tech_if = &match ctx.cfg.read().await.tech.clone() {
                Tech::AfXdp(opts) => opts.if_name.clone(),
            };

            let batch_data_if = batch_read.batches.first().and_then(|b| b.iface.clone());

            let batch_if = batch_read.ovr_opts.as_ref().and_then(|o| o.iface.clone());

            get_ifname_from_src_ip(src_ip)
                .ok()
                .or(batch_data_if)
                .or(batch_if)
                .or(tech_if.clone())
        } else {
            None
        }
    };

    // We need to setup the tech (e.g. create sockets) before we can start the batches.
    if let Err(e) = ctx
        .tech
        .write()
        .await
        .init(ctx.clone(), iface_fb.clone())
        .await
    {
        ctx.logger
            .read()
            .await
            .log_msg(
                LogLevel::Fatal,
                &format!("Failed to setup tech (e.g. create sockets): {}", e),
            )
            .ok();

        process::exit(1);
    }

    ctx.logger
        .read()
        .await
        .log_msg(LogLevel::Info, "Tech initialized. Starting batches...")
        .ok();

    // We need to create an atomic bool to signal halting execution in batch threads.
    let running = Arc::new(AtomicBool::new(true));
    let running_batch = running.clone();

    // Start batches.
    let batch_hdl = tokio::spawn({
        let ctx = ctx.clone();
        let iface_fb = iface_fb.clone();

        async move {
            match ctx
                .batch
                .read()
                .await
                .clone()
                .start_batches(ctx.clone(), running_batch.clone(), iface_fb.clone())
                .await
            {
                Ok(_) => {
                    ctx.logger
                        .read()
                        .await
                        .log_msg(LogLevel::Info, "Batches completed successfully.")
                        .ok();
                }
                Err(e) => {
                    ctx.logger
                        .read()
                        .await
                        .log_msg(LogLevel::Error, &format!("Batch execution failed: {}", e))
                        .ok();

                    process::exit(1);
                }
            }
        }
    });

    let watcher_hdl = match ctx.cli.read().await.clone().args.watch {
        true => {
            ctx.logger
                .read()
                .await
                .log_msg(LogLevel::Info, "Starting watcher...")
                .ok();

            let ctx = ctx.clone();
            let running = running.clone();
            let iface_fb = iface_fb.clone();

            Some(tokio::spawn(async move {
                if let Err(e) = watcher::run::watcher_run(
                    ctx,
                    running,
                    iface_fb.unwrap_or_else(|| "unknown".to_string()),
                )
                .await
                {
                    eprintln!("Watcher failed: {}", e);
                }
            }))
        }
        false => None,
    };

    // Setup signal.
    tokio::select! {
        res = batch_hdl => {
            if let Err(e) = res {
                ctx.logger
                    .read()
                    .await
                    .log_msg(LogLevel::Error, &format!("Batch task failed: {}", e))
                    .ok();

                process::exit(1);
            }
        }
        _ = tokio::signal::ctrl_c() => {
            ctx.logger
                .read()
                .await
                .log_msg(LogLevel::Info, "Received Ctrl+C signal. Shutting down...")
                .ok();

            running.store(false, Ordering::Relaxed);

        }
    }

    if let Some(hdl) = watcher_hdl {
        hdl.await.ok();
    }
}
