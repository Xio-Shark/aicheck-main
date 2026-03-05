use std::collections::HashMap;
use std::io::Read;

use aidoc_core::{
    build_pack, normalize_text, EnvironmentSnapshot, OutputLanguage, Pack as AidocPack, RuleHit,
};
use aidoc_output::{render_pack_json, render_pack_md};
use aidoc_probes::{collect_min_snapshot, collect_snapshot, is_elevated, sanitize_snapshot};
use aidoc_redact::redact_text;
use aidoc_signatures::{detect_environment_issues, match_signatures};
use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;

const SECTION_BREAK: &str = "---AIDOC-SECTION-BREAK---";
const EXIT_NO_ISSUE: i32 = 0;
const EXIT_ISSUE_FOUND: i32 = 1;
const EXIT_TOOL_ERROR: i32 = 2;
const EXIT_PERMISSION_REQUIRED: i32 = 3;
const EXIT_LLM_FAILED: i32 = 4;

#[derive(Parser, Debug)]
#[command(name = "aidoc")]
#[command(about = "Read-only elevated diagnostic summarizer")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Paste(CommonOptions),
    Diagnose(DiagnoseOptions),
    Pack(CommonOptions),
    Redact(CommonOptions),
    Explain,
}

#[derive(Args, Debug, Clone)]
struct CommonOptions {
    #[arg(long, value_enum, default_value_t = FormatArg::Md)]
    format: FormatArg,
    #[arg(long, value_enum, default_value_t = OutputLangArg::En)]
    lang_output: OutputLangArg,
    #[arg(long, default_value_t = 400)]
    max_log_lines: usize,
    #[arg(long, value_enum, default_value_t = NetworkArg::On)]
    network: NetworkArg,
    #[arg(long, value_enum, default_value_t = OsArg::Auto)]
    os: OsArg,
    #[arg(long, value_enum, default_value_t = StackArg::Auto)]
    lang: StackArg,
    #[arg(long, value_enum, default_value_t = LlmArg::Off)]
    llm: LlmArg,
    #[arg(long)]
    llm_provider: Option<String>,
    #[arg(long)]
    llm_model: Option<String>,
    #[arg(long)]
    llm_endpoint: Option<String>,
    #[arg(long, default_value_t = false)]
    llm_dry_run: bool,
    #[arg(long, default_value_t = 5000)]
    timeout: u64,
}

#[derive(Args, Debug, Clone)]
struct DiagnoseOptions {
    #[command(flatten)]
    common: CommonOptions,
    #[arg(long, default_value_t = false)]
    elevated: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum FormatArg {
    Md,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputLangArg {
    En,
    Zh,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum NetworkArg {
    On,
    Off,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OsArg {
    Auto,
    Windows,
    Linux,
    Macos,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum StackArg {
    Auto,
    Python,
    Node,
    Java,
    Go,
    Rust,
    Docker,
    Git,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum LlmArg {
    On,
    Off,
}

#[derive(Clone, Debug)]
struct LlmRender {
    provider: String,
    model: String,
    endpoint: String,
    summary: Option<String>,
    payload_preview: String,
}

fn main() {
    let cli = Cli::parse();
    let code = match execute(cli) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("aidoc error: {err}");
            EXIT_TOOL_ERROR
        }
    };

    std::process::exit(code);
}

fn execute(cli: Cli) -> Result<i32> {
    match cli.command {
        Commands::Paste(opts) => run_paste(opts),
        Commands::Diagnose(opts) => run_diagnose(opts),
        Commands::Pack(opts) => run_pack(opts),
        Commands::Redact(opts) => run_redact(opts),
        Commands::Explain => {
            run_explain();
            Ok(0)
        }
    }
}

fn run_paste(opts: CommonOptions) -> Result<i32> {
    let input = read_stdin()?;
    let snapshot = collect_min_snapshot();
    let (pack, hits) = build_log_pack(&input, snapshot, &opts, Vec::new());
    finalize_pack(pack, &hits, &opts, "paste.completed")
}

fn run_diagnose(opts: DiagnoseOptions) -> Result<i32> {
    if opts.elevated && !is_elevated() {
        eprintln!("aidoc: current process is not elevated, please rerun with administrator/root");
        emit_log(
            "diagnose.permission_required",
            None,
            EXIT_PERMISSION_REQUIRED,
        );
        return Ok(EXIT_PERMISSION_REQUIRED);
    }

    let snapshot = collect_snapshot(
        matches!(opts.common.network, NetworkArg::On),
        opts.common.timeout,
    );
    let hits = detect_environment_issues(&snapshot);
    let pack = build_pack(
        "",
        Vec::new(),
        hits.clone(),
        snapshot,
        map_lang(opts.common.lang_output),
    );

    finalize_pack(pack, &hits, &opts.common, "diagnose.completed")
}

fn run_pack(opts: CommonOptions) -> Result<i32> {
    let input = read_stdin()?;
    let (log_part, snapshot) = parse_pack_input(&input)?;
    let env_hits = detect_environment_issues(&snapshot);
    let (pack, hits) = build_log_pack(log_part, snapshot, &opts, env_hits);
    finalize_pack(pack, &hits, &opts, "pack.completed")
}

fn build_log_pack(
    log_part: &str,
    snapshot: EnvironmentSnapshot,
    opts: &CommonOptions,
    extra_hits: Vec<RuleHit>,
) -> (AidocPack, Vec<RuleHit>) {
    let normalized = normalize_text(log_part, opts.max_log_lines);
    let hits = merge_hits(match_signatures(&normalized), extra_hits);
    let redaction = redact_text(&normalized);

    let pack = build_pack(
        &redaction.redacted,
        redaction.items,
        hits.clone(),
        snapshot,
        map_lang(opts.lang_output),
    );

    (pack, hits)
}

fn parse_pack_input(input: &str) -> Result<(&str, EnvironmentSnapshot)> {
    let sections: Vec<&str> = input.split(SECTION_BREAK).collect();
    if sections.len() < 2 {
        return Err(anyhow!(
            "pack requires two stdin sections separated by {}",
            SECTION_BREAK
        ));
    }

    let snapshot = serde_json::from_str::<EnvironmentSnapshot>(sections[1].trim())
        .map(sanitize_snapshot)
        .map_err(|e| anyhow!("invalid snapshot json: {e}"))?;

    Ok((sections[0], snapshot))
}

fn run_redact(opts: CommonOptions) -> Result<i32> {
    let input = read_stdin()?;
    let redaction = redact_text(&input);

    match opts.format {
        FormatArg::Md => print_redact_md(redaction),
        FormatArg::Json => {
            let payload = json!({
                "redacted": redaction.redacted,
                "redactions": redaction.items,
            });
            println!("{}", serde_json::to_string_pretty(&payload)?);
        }
    }

    Ok(0)
}

fn print_redact_md(redaction: aidoc_redact::RedactionResult) {
    println!("{}", redaction.redacted);
    println!();
    println!("# Redaction Report");

    if redaction.items.is_empty() {
        println!("- no redactions");
        return;
    }

    for item in redaction.items {
        println!(
            "- {} -> {} (count: {})",
            item.kind, item.placeholder, item.count
        );
    }
}

fn run_explain() {
    println!("aidoc explain");
    println!();
    println!("This tool only runs read-only probes and never writes files or registry.");
    println!("Allowed actions:");
    println!("1) Read stdin logs");
    println!("2) Match signatures on normalized text");
    println!("3) Redact sensitive values");
    println!("4) Run whitelist commands with timeout and output cap");
    println!("5) Render handover pack to stdout");
    println!("6) Optional LLM summary over redacted pack only");
    println!();
    println!("Pack merge protocol:");
    println!("- section 1: raw error log");
    println!("- separator: {}", SECTION_BREAK);
    println!("- section 2: diagnose snapshot JSON");
}

fn finalize_pack(
    mut pack: AidocPack,
    hits: &[RuleHit],
    opts: &CommonOptions,
    event: &str,
) -> Result<i32> {
    let llm_render = match maybe_apply_llm(opts, &mut pack) {
        Ok(render) => render,
        Err(err) => {
            eprintln!("aidoc llm error: {err}");
            emit_log(
                &format!("{}.llm_failed", event),
                Some(&pack.trace_id),
                EXIT_LLM_FAILED,
            );
            return Ok(EXIT_LLM_FAILED);
        }
    };

    print_pack(&pack, opts.format, llm_render.as_ref())?;
    let code = exit_code_for_hits(hits);
    emit_log(event, Some(&pack.trace_id), code);
    Ok(code)
}

fn exit_code_for_hits(hits: &[RuleHit]) -> i32 {
    if hits.is_empty() {
        EXIT_NO_ISSUE
    } else {
        EXIT_ISSUE_FOUND
    }
}

fn maybe_apply_llm(opts: &CommonOptions, pack: &mut AidocPack) -> Result<Option<LlmRender>> {
    if !llm_requested(opts) {
        return Ok(None);
    }

    #[cfg(not(feature = "llm"))]
    {
        let _ = pack;
        return Err(anyhow!("llm is not enabled in current build"));
    }

    #[cfg(feature = "llm")]
    {
        apply_llm_render(opts, pack).map(Some)
    }
}

#[cfg(feature = "llm")]
fn apply_llm_render(opts: &CommonOptions, pack: &mut AidocPack) -> Result<LlmRender> {
    let request = aidoc_llm::LlmRequestOptions {
        provider: opts.llm_provider.clone(),
        model: opts.llm_model.clone(),
        endpoint: opts.llm_endpoint.clone(),
        api_key: std::env::var("AIDOC_API_KEY").ok(),
        timeout_ms: opts.timeout,
        dry_run: opts.llm_dry_run,
    };

    let execution = futures::executor::block_on(aidoc_llm::summarize_pack(pack, &request))?;
    if let Some(summary) = &execution.summary {
        pack.issue_summary = merge_issue_summary(&pack.issue_summary, summary, &execution.provider);
    }

    Ok(LlmRender {
        provider: execution.provider,
        model: execution.model,
        endpoint: execution.endpoint,
        summary: execution.summary,
        payload_preview: execution.payload_preview,
    })
}

#[cfg(feature = "llm")]
fn merge_issue_summary(current: &str, llm_summary: &str, provider: &str) -> String {
    if current.trim().is_empty() {
        return format!("LLM ({provider}) summary: {}", llm_summary.trim());
    }

    format!(
        "{current}\n\nLLM ({provider}) summary:\n{}",
        llm_summary.trim()
    )
}

fn llm_requested(opts: &CommonOptions) -> bool {
    matches!(opts.llm, LlmArg::On)
}

fn emit_log(event: &str, trace_id: Option<&str>, exit_code: i32) {
    let payload = json!({
        "event": event,
        "trace_id": trace_id.unwrap_or(""),
        "exit_code": exit_code,
    });
    if let Ok(line) = serde_json::to_string(&payload) {
        eprintln!("{line}");
    }
}

fn merge_hits(primary: Vec<RuleHit>, secondary: Vec<RuleHit>) -> Vec<RuleHit> {
    let mut map: HashMap<String, RuleHit> = HashMap::new();

    for hit in primary.into_iter().chain(secondary) {
        let key = hit.id.clone();
        if let Some(existing) = map.get_mut(&key) {
            if hit.confidence > existing.confidence {
                *existing = hit;
                continue;
            }

            existing.evidence.extend(hit.evidence);
            extend_sorted_unique(&mut existing.verify_commands, hit.verify_commands);
            extend_sorted_unique(&mut existing.fix_suggestions_en, hit.fix_suggestions_en);
            extend_sorted_unique(&mut existing.fix_suggestions_zh, hit.fix_suggestions_zh);
        } else {
            map.insert(key, hit);
        }
    }

    let mut values: Vec<RuleHit> = map.into_values().collect();
    values.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));
    values
}

fn extend_sorted_unique(target: &mut Vec<String>, source: Vec<String>) {
    target.extend(source);
    target.sort();
    target.dedup();
}

fn print_pack(pack: &AidocPack, format: FormatArg, llm: Option<&LlmRender>) -> Result<()> {
    match format {
        FormatArg::Md => {
            println!("{}", render_pack_md(pack));
            if let Some(info) = llm {
                print_llm_md(info);
            }
        }
        FormatArg::Json => print_pack_json(pack, llm)?,
    }

    Ok(())
}

fn print_pack_json(pack: &AidocPack, llm: Option<&LlmRender>) -> Result<()> {
    if let Some(info) = llm {
        let payload = json!({
            "pack": pack,
            "llm": {
                "provider": info.provider,
                "model": info.model,
                "endpoint": info.endpoint,
                "summary": info.summary,
                "payload_preview": info.payload_preview,
            }
        });
        println!("{}", serde_json::to_string_pretty(&payload)?);
        return Ok(());
    }

    println!("{}", render_pack_json(pack)?);
    Ok(())
}

fn print_llm_md(info: &LlmRender) {
    println!("\n## 8) LLM Summary");
    println!("- Provider: {}", info.provider);
    println!("- Model: {}", info.model);
    println!("- Endpoint: {}", info.endpoint);

    match &info.summary {
        Some(summary) => println!("\n{}", summary.trim()),
        None => println!("- Mode: dry-run (payload preview only)"),
    }

    println!("\n### LLM Payload Preview");
    println!("```json");
    println!("{}", info.payload_preview);
    println!("```");
}

fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    std::io::stdin().read_to_string(&mut buf)?;
    if buf.trim().is_empty() {
        return Err(anyhow!("stdin is empty"));
    }
    Ok(buf)
}

fn map_lang(lang: OutputLangArg) -> OutputLanguage {
    match lang {
        OutputLangArg::En => OutputLanguage::En,
        OutputLangArg::Zh => OutputLanguage::Zh,
    }
}
