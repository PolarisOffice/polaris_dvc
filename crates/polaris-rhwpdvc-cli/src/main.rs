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

/// Which conditional-field set to emit per violation. Mirrors upstream
/// `DVCOutputOption` enum (ExportInterface.h:26). Upstream also exposes
/// these via single-letter flags (`-d` Default, `-o` AllOption, `-t`
/// Table, `-i` TableDetail, `-p` Shape, `-y` Style, `-k` Hyperlink) but
/// those collide with spec-file loading (`-t <SPEC>`); we expose one
/// clean long flag instead.
#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "kebab-case")]
enum OutputOptionArg {
    Default,
    All,
    Table,
    TableDetail,
    Style,
    Shape,
    Hyperlink,
}

impl OutputOptionArg {
    fn to_core(self) -> polaris_rhwpdvc_core::output::OutputOption {
        use polaris_rhwpdvc_core::output::OutputOption as O;
        match self {
            Self::Default => O::Default,
            Self::All => O::AllOption,
            Self::Table => O::Table,
            Self::TableDetail => O::TableDetail,
            Self::Style => O::Style,
            Self::Shape => O::Shape,
            Self::Hyperlink => O::Hyperlink,
        }
    }
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

    /// Equivalent to `--format=xml`. Available in the default
    /// (Extended) profile; under `--dvc-strict` this exits 2 to match
    /// upstream DVC, which never implemented XML output.
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

    /// Enable KS X 6101 XSD conformance checks (JID 13000-13999).
    /// Off by default because the bundled schema model is a bootstrap
    /// subset — on unfamiliar elements it produces many findings.
    /// Flip on to audit document structure against the standard.
    #[arg(long = "enable-schema")]
    enable_schema: bool,

    /// Output option — which conditional fields are emitted per
    /// violation. Mirrors upstream `DVCOutputOption`. Default: `all`.
    #[arg(long = "output-option", value_enum, default_value = "all")]
    output_option: OutputOptionArg,

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
    // Upstream DVC's `-x` / `--format=xml` is unimplemented
    // (`CommandParser.cpp` returns `NotYet`). In `--dvc-strict` we match
    // that behavior exactly — XML requests exit 2 with the upstream
    // message — so output stays byte-compatible. In the default
    // (Extended) profile we go beyond upstream and emit our own XML.
    if matches!(format, Format::Xml) && cli.dvc_strict {
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
        enable_schema: cli.enable_schema,
    };
    let report = polaris_rhwpdvc_core::engine::validate(&doc, &spec, &opts);

    let option = cli.output_option.to_core();
    let body: String = match format {
        Format::Json => {
            let payload = report.to_json_value(option);
            serde_json::to_string_pretty(&payload).expect("serialize report")
        }
        Format::Xml => {
            // Gated above: only reachable when NOT in --dvc-strict.
            report.to_xml_string(option)
        }
    };
    if let Some(path) = cli.file {
        if let Err(e) = std::fs::write(&path, body.as_bytes()) {
            eprintln!("polaris-rhwpdvc: failed to write output: {e}");
            return ExitCode::from(3);
        }
    } else {
        let stdout = std::io::stdout();
        let mut lock = stdout.lock();
        let trailer: &[u8] = if matches!(format, Format::Xml) {
            b""
        } else {
            b"\n"
        };
        if lock.write_all(body.as_bytes()).is_err() || lock.write_all(trailer).is_err() {
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
