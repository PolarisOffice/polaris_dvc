//! polaris CLI — DVC-compatible flag surface.
//!
//! Flag mapping (upstream → polaris) is documented in `docs/cli-compat.md`
//! and exercised by the golden-regression tests under `testdata/golden/`.

use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
enum Format {
    Json,
    Xml,
}

#[derive(Parser, Debug)]
#[command(
    name = "polaris-rhwpdvc",
    version,
    about = "HWPX validator (DVC-compatible). See docs/cli-compat.md."
)]
struct Cli {
    /// Equivalent to `--format=json` (default).
    #[arg(short = 'j', long_help = "Emit JSON (default).")]
    json: bool,

    /// Equivalent to `--format=xml` (not implemented yet).
    #[arg(short = 'x')]
    xml: bool,

    /// Output format. Overrides -j/-x when given explicitly.
    #[arg(long, value_enum)]
    format: Option<Format>,

    /// Write output to a file instead of stdout.
    #[arg(long = "file", value_name = "PATH")]
    file: Option<PathBuf>,

    /// Stop at first violation.
    #[arg(short = 's', long = "simple")]
    simple: bool,

    /// Report all violations (default).
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Rule spec JSON path.
    #[arg(short = 't', value_name = "SPEC")]
    spec: Option<PathBuf>,

    /// DVC strict mode — only emit violations for JIDs upstream
    /// `DVC.exe` actually validates. Default (off) is the "Extended"
    /// profile which also fires on JIDs upstream leaves as no-op
    /// (margin-*, bgfill-*, bggradation-*, caption-*, etc.). Use this
    /// flag when byte-compat with DVC.exe matters.
    #[arg(long = "dvc-strict")]
    dvc_strict: bool,

    /// HWPX document path, or `-` for stdin.
    #[arg(value_name = "INPUT")]
    input: Option<String>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    let format = match (cli.format.clone(), cli.xml) {
        (Some(f), _) => f,
        (None, true) => Format::Xml,
        (None, false) => Format::Json,
    };
    if matches!(format, Format::Xml) {
        eprintln!("polaris-rhwpdvc: --format=xml is not yet implemented");
        return ExitCode::from(2);
    }

    let Some(input) = cli.input else {
        eprintln!("polaris-rhwpdvc: input path is required");
        return ExitCode::from(2);
    };

    let doc_bytes = match read_input(&input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("polaris-rhwpdvc: failed to read input: {e}");
            return ExitCode::from(3);
        }
    };

    let doc = match polaris_rhwpdvc_format::parse(&doc_bytes) {
        Ok(polaris_rhwpdvc_format::Document::Hwpx(d)) => d,
        Err(e) => {
            eprintln!("polaris-rhwpdvc: parse error: {e}");
            return ExitCode::from(3);
        }
    };

    let spec = match cli.spec {
        Some(p) => match polaris_rhwpdvc_core::rules::loader::load_spec_from_path(&p) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("polaris-rhwpdvc: failed to load spec: {e}");
                return ExitCode::from(2);
            }
        },
        None => polaris_rhwpdvc_core::rules::schema::RuleSpec::default(),
    };

    let opts = polaris_rhwpdvc_core::engine::EngineOptions {
        stop_on_first: cli.simple,
        profile: if cli.dvc_strict {
            polaris_rhwpdvc_core::engine::CheckProfile::DvcStrict
        } else {
            polaris_rhwpdvc_core::engine::CheckProfile::Extended
        },
    };
    let report = polaris_rhwpdvc_core::engine::validate(&doc, &spec, &opts);

    let payload = report.to_json_value(polaris_rhwpdvc_core::output::OutputOption::AllOption);
    let json = serde_json::to_string_pretty(&payload).expect("serialize report");
    if let Some(path) = cli.file {
        if let Err(e) = std::fs::write(&path, json.as_bytes()) {
            eprintln!("polaris-rhwpdvc: failed to write output: {e}");
            return ExitCode::from(3);
        }
    } else {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        if lock.write_all(json.as_bytes()).is_err() || lock.write_all(b"\n").is_err() {
            return ExitCode::from(3);
        }
    }

    if report.violations.is_empty() {
        ExitCode::from(0)
    } else {
        ExitCode::from(1)
    }
}

fn read_input(path: &str) -> std::io::Result<Vec<u8>> {
    if path == "-" {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)?;
        Ok(buf)
    } else {
        std::fs::read(path)
    }
}
