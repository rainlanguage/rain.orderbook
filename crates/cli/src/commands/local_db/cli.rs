use super::pipeline::runner::{ProducerJobFailure, ProducerRunReport};
use crate::commands::local_db::pipeline::runner::ProducerRunner;
use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, Parser)]
#[command(about = "Run the producer pipeline across all orderbooks in settings.yaml")]
pub struct RunPipeline {
    #[clap(
        long,
        help = "Full YAML document that configures the local DB pipeline",
        value_name = "YAML"
    )]
    pub settings_yaml: String,

    #[clap(
        long,
        help = "HyperRPC API token used to fetch logs during the run",
        value_name = "TOKEN"
    )]
    pub api_token: String,

    #[clap(
        long,
        help = "Output directory where per-orderbook SQLite databases and dumps are written",
        value_name = "PATH",
        default_value = "./local-db"
    )]
    pub out_root: PathBuf,

    #[clap(
        long,
        help = "Base URL for published dumps (e.g., https://example.com/releases)",
        value_name = "URL",
        value_parser = clap::value_parser!(Url)
    )]
    pub release_base_url: Url,
}

impl RunPipeline {
    pub async fn execute(self) -> Result<()> {
        println!("Starting producer pipeline run");

        let RunPipeline {
            settings_yaml,
            api_token,
            out_root,
            release_base_url,
        } = self;

        std::fs::create_dir_all(&out_root)?;

        let runner = ProducerRunner::new(settings_yaml, out_root, release_base_url, api_token)?;
        let report = runner.run().await?;

        render_report(&report);

        if report.failures.is_empty() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("one or more producer jobs failed"))
        }
    }
}

fn render_report(report: &ProducerRunReport) {
    let mut stdout = io::stdout();
    let _ = render_report_to(report, &mut stdout);
}

fn render_report_to<W: Write>(report: &ProducerRunReport, writer: &mut W) -> io::Result<()> {
    if report.successes.is_empty() {
        writeln!(writer, "No producer jobs completed successfully.")?;
    } else {
        writeln!(
            writer,
            "Producer pipeline completed {} successful job(s):",
            report.successes.len()
        )?;
        for outcome in &report.successes {
            let target = &outcome.outcome.target;
            match &outcome.exported_dump {
                Some(export) => {
                    writeln!(
                        writer,
                        "- chain {} orderbook {:#x}: start {} → target {} | logs {} | events {} | dump {} (end block {}, hash {}, time {})",
                        target.chain_id,
                        target.orderbook_address,
                        outcome.outcome.start_block,
                        outcome.outcome.target_block,
                        outcome.outcome.fetched_logs,
                        outcome.outcome.decoded_events,
                        export.dump_path.display(),
                        export.end_block,
                        export.end_block_hash,
                        export.end_block_time_ms,
                    )?;
                }
                None => {
                    writeln!(
                        writer,
                        "- chain {} orderbook {:#x}: start {} → target {} | logs {} | events {} | dump <none>",
                        target.chain_id,
                        target.orderbook_address,
                        outcome.outcome.start_block,
                        outcome.outcome.target_block,
                        outcome.outcome.fetched_logs,
                        outcome.outcome.decoded_events,
                    )?;
                }
            }
        }
    }

    if report.failures.is_empty() {
        writeln!(writer, "All producer jobs completed successfully.")?;
    } else {
        writeln!(
            writer,
            "{} job(s) failed during the producer run:",
            report.failures.len()
        )?;
        for failure in &report.failures {
            render_failure_to(failure, writer)?;
        }
    }

    Ok(())
}

fn render_failure_to<W: Write>(failure: &ProducerJobFailure, writer: &mut W) -> io::Result<()> {
    match (failure.chain_id, failure.orderbook_address) {
        (Some(chain_id), Some(address)) => writeln!(
            writer,
            "- chain {} orderbook {:#x} failed: {}",
            chain_id, address, failure.error
        ),
        (Some(chain_id), None) => writeln!(
            writer,
            "- chain {} orderbook <unknown> failed: {}",
            chain_id, failure.error
        ),
        (None, Some(address)) => writeln!(
            writer,
            "- chain <unknown> orderbook {:#x} failed: {}",
            address, failure.error
        ),
        (None, None) => writeln!(
            writer,
            "- job failed before identifying target: {}",
            failure.error
        ),
    }
}

#[cfg(test)]
mod tests {
    use crate::commands::local_db::pipeline::runner::{ExportMetadata, ProducerOutcome};

    use super::*;
    use alloy::primitives::address;
    use rain_orderbook_common::local_db::pipeline::engine::SyncInputs;
    use rain_orderbook_common::local_db::pipeline::runner::utils::RunnerTarget;
    use rain_orderbook_common::local_db::pipeline::{
        FinalityConfig, SyncConfig, SyncOutcome, TargetKey, WindowOverrides,
    };
    use rain_orderbook_common::local_db::{FetchConfig, LocalDbError};

    #[test]
    fn default_out_root_is_local_db() {
        let command = RunPipeline::parse_from([
            "sync",
            "--settings-yaml",
            "test",
            "--api-token",
            "token",
            "--release-base-url",
            "https://example.com/releases",
        ]);
        assert_eq!(command.out_root, PathBuf::from("./local-db"));
    }

    fn sample_outcome(chain_id: u32) -> ProducerOutcome {
        let target = TargetKey {
            chain_id,
            orderbook_address: address!("0000000000000000000000000000000000000a11"),
        };
        let fetch = FetchConfig::new(10, 5, 2, 1).unwrap();
        let sync_config = SyncConfig {
            deployment_block: 100,
            fetch,
            finality: FinalityConfig { depth: 12 },
            window_overrides: WindowOverrides::default(),
        };
        let inputs = SyncInputs {
            target: target.clone(),
            metadata_rpcs: Vec::new(),
            cfg: sync_config,
            dump_str: None,
            manifest_end_block: 1,
        };
        let runner_target = RunnerTarget {
            orderbook_key: "test".to_string(),
            manifest_url: "https://example.com/manifest.yaml".parse().unwrap(),
            network_key: "anvil".to_string(),
            inputs,
        };

        ProducerOutcome {
            outcome: SyncOutcome {
                target: runner_target.inputs.target.clone(),
                start_block: 200,
                target_block: 400,
                fetched_logs: 123,
                decoded_events: 456,
            },
            exported_dump: Some(ExportMetadata {
                dump_path: PathBuf::from(format!(
                    "./local-db/{}/{}-{}.sql.gz",
                    chain_id, chain_id, runner_target.inputs.target.orderbook_address
                )),
                end_block: 400,
                end_block_hash: "0xdeadbeef".to_string(),
                end_block_time_ms: 1_700_000_000,
            }),
        }
    }

    #[test]
    fn render_report_to_writes_success_summary() {
        let report = ProducerRunReport {
            successes: vec![sample_outcome(42161)],
            failures: vec![],
        };
        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("1 successful job"));
        assert!(output.contains("chain 42161"));
        assert!(output.contains("start 200 → target 400"));
        assert!(output.contains("logs 123"));
        assert!(output.contains("events 456"));
        assert!(output.contains("All producer jobs completed successfully."));
    }

    #[test]
    fn render_report_to_handles_missing_dump() {
        let mut outcome = sample_outcome(10);
        outcome.exported_dump = None;
        let report = ProducerRunReport {
            successes: vec![outcome],
            failures: vec![],
        };

        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("dump <none>"));
    }

    #[test]
    fn render_report_to_lists_failures() {
        let orderbook_address = address!("0000000000000000000000000000000000000fA1");
        let failure = ProducerJobFailure {
            chain_id: Some(1),
            orderbook_address: Some(orderbook_address),
            error: LocalDbError::CustomError("oh no".into()),
        };
        let report = ProducerRunReport {
            successes: Vec::new(),
            failures: vec![failure],
        };

        let mut buffer = Vec::new();
        render_report_to(&report, &mut buffer).expect("render succeeds");

        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("No producer jobs completed successfully."));
        assert!(output.contains("1 job(s) failed"));
        assert!(output.contains(&format!("chain {} orderbook {:#x}", 1, orderbook_address)));
        assert!(output.contains("oh no"));
    }

    #[test]
    fn render_failure_to_handles_unknowns() {
        let failure = ProducerJobFailure {
            chain_id: None,
            orderbook_address: None,
            error: LocalDbError::CustomError("boom".into()),
        };
        let mut buffer = Vec::new();
        render_failure_to(&failure, &mut buffer).expect("render succeeds");
        let output = String::from_utf8(buffer).expect("utf8");
        assert!(output.contains("job failed before identifying target"));
    }
}
