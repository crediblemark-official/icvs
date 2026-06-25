use std::collections::HashMap;
use std::path::PathBuf;

use clap::{Parser as ClapParser, Subcommand, ValueEnum};

use icvs::error::{IcvsError, Result};
use icvs::exporter;
use icvs::resolver;
use icvs::validator;
use icvs::md_converter;
use icvs::template;
use icvs::agent_format;

fn load_document(path: &PathBuf) -> Result<icvs::ast::Document> {
    resolver::resolve_file(path)
}

#[derive(ClapParser)]
#[command(name = "icvs", version, about = "InstructCanvas - DAG-based instruction format for agentic AI tools")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Debug, Clone)]
enum AgentFormat {
    Claude,
    OpenAI,
    Json,
}

#[derive(Subcommand)]
enum Commands {
    /// Validate one or more .icvs files
    Validate {
        /// Path to .icvs file
        file: PathBuf,
        /// Enable strict mode (reject unknown attributes)
        #[arg(long)]
        strict: bool,
    },
    /// Export instructions for a specific target
    Export {
        /// Path to .icvs file
        file: PathBuf,
        /// Target name (e.g., claude, copilot, cursor)
        #[arg(long)]
        target: String,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Visualize the instruction graph as DOT
    Visualize {
        /// Path to .icvs file
        file: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Merge all nodes into a single Markdown document
    Merge {
        /// Path to .icvs file
        file: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Convert Markdown to .icvs format
    MdToIcvs {
        /// Path to Markdown file
        file: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Convert .icvs to clean Markdown
    IcvsToMd {
        /// Path to .icvs file
        file: PathBuf,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Export .icvs as AI agent tool definitions (Claude/OpenAI/JSON)
    Convert {
        /// Path to .icvs file
        file: PathBuf,
        /// Target format
        #[arg(long, value_enum)]
        target: AgentFormat,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Apply template variables to an .icvs file
    Template {
        /// Path to .icvs file
        file: PathBuf,
        /// Variable replacements in key=value format (can be repeated)
        #[arg(long = "var", short = 'D')]
        vars: Vec<String>,
        /// Output file (defaults to stdout)
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Run benchmark comparing .icvs vs Markdown
    Benchmark {
        /// Path to .icvs file
        file: PathBuf,
        /// Number of iterations
        #[arg(long, default_value = "100")]
        iterations: usize,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Validate { file, strict: _ } => cmd_validate(file),
        Commands::Export { file, target, output } => cmd_export(file, target, output),
        Commands::Visualize { file, output } => cmd_visualize(file, output),
        Commands::Merge { file, output } => cmd_merge(file, output),
        Commands::MdToIcvs { file, output } => cmd_md_to_icvs(file, output),
        Commands::IcvsToMd { file, output } => cmd_icvs_to_md(file, output),
        Commands::Convert { file, target, output } => cmd_convert(file, target.clone(), output),
        Commands::Template { file, vars, output } => cmd_template(file, vars, output),
        Commands::Benchmark { file, iterations } => cmd_benchmark(file, *iterations),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn cmd_validate(path: &PathBuf) -> Result<()> {
    let doc = load_document(path)?;
    let report = validator::validate(&doc)?;

    if !report.is_valid {
        eprintln!("❌ Invalid: {}", path.display());
        for err in &report.errors {
            eprintln!("   Error: {}", err);
        }
        return Err(IcvsError::Validation {
            message: format!("File has {} error(s)", report.errors.len()),
        });
    }

    println!("✅ Valid: {}", path.display());
    println!("   Nodes: {}", report.node_count);
    println!("   Edges: {}", report.edge_count);

    if !report.warnings.is_empty() {
        println!("\nWarnings:");
        for w in &report.warnings {
            println!("  ⚠ {}", w);
        }
    }

    if report.orphan_nodes.is_empty() {
        println!("   Orphan nodes: none");
    } else {
        println!("   Orphan nodes: {}", report.orphan_nodes.join(", "));
    }

    Ok(())
}

fn cmd_export(path: &PathBuf, target: &str, output: &Option<PathBuf>) -> Result<()> {
    let doc = load_document(path)?;
    let report = validator::validate(&doc)?;

    if !report.is_valid {
        return Err(IcvsError::Validation {
            message: format!("File is invalid ({} errors)", report.errors.len()),
        });
    }

    let markdown = exporter::export_markdown(&doc, target)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &markdown)
                .map_err(|e| IcvsError::Io {
                    path: out_path.clone(),
                    message: e.to_string(),
                })?;
            println!("Exported to: {}", out_path.display());
        }
        None => {
            println!("{}", markdown);
        }
    }

    Ok(())
}

fn cmd_visualize(path: &PathBuf, output: &Option<PathBuf>) -> Result<()> {
    let doc = load_document(path)?;
    let dot = exporter::export_dot(&doc)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &dot)
                .map_err(|e| IcvsError::Io {
                    path: out_path.clone(),
                    message: e.to_string(),
                })?;
            println!("DOT graph written to: {}", out_path.display());
        }
        None => {
            println!("{}", dot);
        }
    }

    Ok(())
}

fn cmd_merge(path: &PathBuf, output: &Option<PathBuf>) -> Result<()> {
    let doc = load_document(path)?;
    let report = validator::validate(&doc)?;

    if !report.is_valid {
        return Err(IcvsError::Validation {
            message: format!("File is invalid ({} errors)", report.errors.len()),
        });
    }

    let merged = exporter::export_markdown_merge(&doc)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &merged)
                .map_err(|e| IcvsError::Io {
                    path: out_path.clone(),
                    message: e.to_string(),
                })?;
            println!("Merged Markdown written to: {}", out_path.display());
        }
        None => {
            println!("{}", merged);
        }
    }

    Ok(())
}

fn cmd_md_to_icvs(path: &PathBuf, output: &Option<PathBuf>) -> Result<()> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| IcvsError::Io { path: path.clone(), message: e.to_string() })?;
    let icvs = md_converter::md_to_icvs(&content)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &icvs)
                .map_err(|e| IcvsError::Io { path: out_path.clone(), message: e.to_string() })?;
            println!("Written to: {}", out_path.display());
        }
        None => println!("{}", icvs),
    }

    Ok(())
}

fn cmd_icvs_to_md(path: &PathBuf, output: &Option<PathBuf>) -> Result<()> {
    let doc = load_document(path)?;
    let _report = validator::validate(&doc)?;
    let md = md_converter::icvs_to_md(&doc)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &md)
                .map_err(|e| IcvsError::Io { path: out_path.clone(), message: e.to_string() })?;
            println!("Written to: {}", out_path.display());
        }
        None => println!("{}", md),
    }

    Ok(())
}

fn cmd_convert(path: &PathBuf, target: AgentFormat, output: &Option<PathBuf>) -> Result<()> {
    let doc = load_document(path)?;
    let af = match target {
        AgentFormat::Claude => agent_format::AgentFormat::Claude,
        AgentFormat::OpenAI => agent_format::AgentFormat::OpenAI,
        AgentFormat::Json => agent_format::AgentFormat::GenericJson,
    };
    let json = agent_format::export_agent_format(&doc, af)?;

    match output {
        Some(out_path) => {
            std::fs::write(out_path, &json)
                .map_err(|e| IcvsError::Io { path: out_path.clone(), message: e.to_string() })?;
            println!("Written to: {}", out_path.display());
        }
        None => println!("{}", json),
    }

    Ok(())
}

fn cmd_template(path: &PathBuf, vars: &[String], output: &Option<PathBuf>) -> Result<()> {
    let mut vars_map = HashMap::new();
    for v in vars {
        if let Some(eq_pos) = v.find('=') {
            let key = v[..eq_pos].trim().to_string();
            let val = v[eq_pos + 1..].trim().to_string();
            vars_map.insert(key, val);
        } else {
            return Err(IcvsError::Validation {
                message: format!("Invalid variable format '{}'. Use KEY=VALUE", v),
            });
        }
    }

    let mut doc = load_document(path)?;
    let report = validator::validate(&doc)?;
    if !report.is_valid {
        return Err(IcvsError::Validation {
            message: format!("File is invalid ({} errors)", report.errors.len()),
        });
    }

    template::apply_template(&mut doc, &vars_map, true)?;

    // Re-serialize to .icvs format
    let serialized = icvs_to_string(&doc);
    match output {
        Some(out_path) => {
            std::fs::write(out_path, &serialized)
                .map_err(|e| IcvsError::Io { path: out_path.clone(), message: e.to_string() })?;
            println!("Written to: {}", out_path.display());
        }
        None => println!("{}", serialized),
    }

    Ok(())
}

fn icvs_to_string(doc: &icvs::ast::Document) -> String {
    let mut out = String::new();
    if let Some(ref name) = doc.project_name {
        out.push_str(&format!("#project: \"{}\"\n", name));
    }
    for inc in &doc.includes {
        out.push_str(&format!("[include: \"{}\"]\n", inc));
    }
    for node in doc.nodes.values() {
        out.push_str(&format!("\n[node: {}]\n", node.id));
        out.push_str(&format!("  type = {}\n", node.node_type.as_str()));
        if let Some(ref c) = node.content {
            out.push_str(&format!("  content = \"{}\"\n", c));
        }
        if let Some(ref s) = node.severity {
            out.push_str(&format!("  severity = {}\n", s.as_str()));
        }
        if let Some(ref t) = node.trigger_on {
            out.push_str(&format!("  trigger_on = {}\n", t.as_str()));
        }
        if let Some(ref cond) = node.condition {
            out.push_str(&format!("  if = ${} {} \"{}\"\n", cond.variable, cond.operator, cond.value));
            if !cond.then_node.is_empty() {
                out.push_str(&format!("  then = -> {}\n", cond.then_node));
            }
            if let Some(ref else_node) = cond.else_node {
                out.push_str(&format!("  else = -> {}\n", else_node));
            }
        }
    }
    for edge in &doc.edges {
        out.push_str(&format!("\n[edge: {} -> {}]\n", edge.source, edge.target));
    }
    for target in doc.targets.values() {
        out.push_str(&format!("\n[target: {}]\n", target.name));
        if let Some(ref resolve) = target.resolve {
            let list: Vec<String> = resolve.iter().map(|s| format!("\"{}\"", s)).collect();
            out.push_str(&format!("  resolve = [{}]\n", list.join(", ")));
        }
        if let Some(ref ignore) = target.ignore {
            let list: Vec<String> = ignore.iter().map(|s| format!("\"{}\"", s)).collect();
            out.push_str(&format!("  ignore = [{}]\n", list.join(", ")));
        }
    }
    out
}

fn cmd_benchmark(path: &PathBuf, iterations: usize) -> Result<()> {
    // Read file once
    let content = std::fs::read_to_string(path)
        .map_err(|e| IcvsError::Io { path: path.clone(), message: e.to_string() })?;

    // Benchmark .icvs parse
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _doc = icvs::parser::parse_document(&content)?;
    }
    let parse_time = start.elapsed();

    let doc = icvs::parser::parse_document(&content)?;
    let report = validator::validate(&doc)?;
    let node_count = report.node_count;
    let edge_count = report.edge_count;

    // Benchmark .icvs → Markdown conversion (simulates markdown parse cost)
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = exporter::export_markdown_merge(&doc)?;
    }
    let export_time = start.elapsed();

    // Benchmark Markdown → .icvs (simulate bidirectional)
    let md = exporter::export_markdown_merge(&doc)?;
    let start = std::time::Instant::now();
    for _ in 0..iterations {
        let _ = md_converter::md_to_icvs(&md)?;
    }
    let reverse_time = start.elapsed();

    // Token ratio analysis
    let token_ratio = content.len() as f64 / md.len() as f64;
    let info_ratio = if content.len() > 0 {
        (node_count + edge_count) as f64 / content.len() as f64 * 1000.0
    } else {
        0.0
    };

    println!("═══ InstructCanvas Benchmark ═══");
    println!("  File:              {}", path.display());
    println!("  Iterations:        {}", iterations);
    println!("  Nodes:             {}", node_count);
    println!("  Edges:             {}", edge_count);
    println!("");
    println!("─── Parse Performance ───");
    println!("  .icvs parse:       {:?} ({} μs/op)", parse_time, parse_time.as_micros() / iterations as u128);
    println!("  → Markdown export: {:?} ({} μs/op)", export_time, export_time.as_micros() / iterations as u128);
    println!("  ← .icvs from MD:   {:?} ({} μs/op)", reverse_time, reverse_time.as_micros() / iterations as u128);
    println!("");
    println!("─── Structural Efficiency ───");
    println!("  .icvs size:        {} bytes", content.len());
    println!("  Markdown size:     {} bytes", md.len());
    if token_ratio < 1.0 { println!("  Compression:       {:.2}× (icvs is smaller than MD)", 1.0 / token_ratio.max(0.001)); }
    else { println!("  Expansion:         {:.2}× (icvs is larger than MD)", token_ratio); }
    println!("  Info density:      {:.2} nodes+edges per KB", info_ratio);
    println!("");
    println!("─── DAG Metrics ───");
    let sorted = icvs::validator::topological_sort(&doc).ok();
    if let Some(ref order) = sorted {
        println!("  Topo depth:        {}", order.len());
    }

    let mut in_degrees: Vec<usize> = doc.nodes.values()
        .map(|n| doc.edges.iter().filter(|e| e.target == n.id).count())
        .collect();
    in_degrees.sort();
    let max_in = in_degrees.last().unwrap_or(&0);
    println!("  Max fan-in:        {}", max_in);

    let mut out_degrees: Vec<usize> = doc.nodes.values()
        .map(|n| doc.edges.iter().filter(|e| e.source == n.id).count())
        .collect();
    out_degrees.sort();
    let max_out = out_degrees.last().unwrap_or(&0);
    println!("  Max fan-out:       {}", max_out);

    println!("");
    println!("═══ END ═══");

    Ok(())
}
