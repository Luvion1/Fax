//! faxc-drv - Compiler Driver
//!
//! Driver utama yang mengkoordinasikan seluruh tahapan kompilasi.

use faxc_gen::{CodeGenError, LlvmBackend};
use faxc_lex::Lexer;
use faxc_lir::lower_mir_to_lir;
use faxc_lir::opt::optimize_function as optimize_lir;
use faxc_mir::lower_hir_function;
use faxc_mir::opt::optimize_function as optimize_mir;
use faxc_par::Parser;
use faxc_sem::{Item as HirItem, SemanticAnalyzer, TypeContext};
use faxc_util::{DefIdGenerator, Handler};
use std::env;
use std::path::PathBuf;

/// Configuration untuk compiler
#[derive(Debug, Clone)]
pub struct Config {
    pub input_files: Vec<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub target: String,
    pub emit: EmitType,
    pub verbose: bool,
    pub incremental: bool,
    pub help: bool,
    pub version: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitType {
    Tokens,
    Ast,
    Hir,
    Mir,
    Lir,
    LlvmIr,
    Asm,
    Object,
    Exe,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            input_files: Vec::new(),
            output_file: None,
            target: default_target(),
            emit: EmitType::Exe,
            verbose: false,
            incremental: false,
            help: false,
            version: false,
        }
    }
}

/// Parse command line arguments
pub fn parse_args() -> Result<Config, String> {
    let args: Vec<String> = env::args().collect();
    let mut config = Config::default();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "--help" || arg == "-h" {
            config.help = true;
            return Ok(config);
        } else if arg == "--version" || arg == "-V" {
            config.version = true;
            return Ok(config);
        } else if arg == "--verbose" || arg == "-v" {
            config.verbose = true;
        } else if arg == "--output" || arg == "-o" {
            if i + 1 >= args.len() {
                return Err("Missing argument for -o".to_string());
            }
            i += 1;
            config.output_file = Some(PathBuf::from(&args[i]));
        } else if arg == "--target" {
            if i + 1 >= args.len() {
                return Err("Missing argument for --target".to_string());
            }
            i += 1;
            config.target = args[i].clone();
        } else if arg == "--emit" {
            if i + 1 >= args.len() {
                return Err("Missing argument for --emit".to_string());
            }
            i += 1;
            config.emit = match args[i].as_str() {
                "tokens" => EmitType::Tokens,
                "ast" => EmitType::Ast,
                "hir" => EmitType::Hir,
                "mir" => EmitType::Mir,
                "lir" => EmitType::Lir,
                "llvm-ir" => EmitType::LlvmIr,
                "asm" => EmitType::Asm,
                "object" => EmitType::Object,
                "exe" => EmitType::Exe,
                _ => return Err(format!("Unknown emit type: {}", args[i])),
            };
        } else if arg.starts_with('-') {
            return Err(format!("Unknown option: {}", arg));
        } else {
            config.input_files.push(PathBuf::from(arg));
        }
        i += 1;
    }

    Ok(config)
}

/// Print help message
pub fn print_help() {
    println!("Fax Compiler v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Usage: faxc [OPTIONS] <input files>");
    println!();
    println!("Options:");
    println!("  -h, --help           Print this help message");
    println!("  -V, --version        Print version information");
    println!("  -v, --verbose        Enable verbose output");
    println!("  -o, --output <FILE>  Specify output file");
    println!("  --target <TARGET>    Target triple (default: x86_64-unknown-linux-gnu)");
    println!("  --emit <TYPE>        Output type: tokens, ast, hir, mir, lir, asm, llvm-ir, exe");
    println!();
    println!("Examples:");
    println!("  faxc hello.fax              Compile hello.fax to executable");
    println!("  faxc -o hello hello.fax     Compile with custom output name");
    println!("  faxc -v hello.fax           Compile with verbose output");
}

/// Print version
pub fn print_version() {
    println!("faxc {}", env!("CARGO_PKG_VERSION"));
}

/// Session kompilasi
pub struct Session {
    pub config: Config,
    pub sources: SourceMap,
    pub diagnostics: Handler,
    pub def_id_gen: DefIdGenerator,
}

impl Session {
    pub fn new(config: Config) -> Result<Self, CompileError> {
        let mut sources = SourceMap::new();
        let diagnostics = Handler::new();
        let def_id_gen = DefIdGenerator::new();

        for path in &config.input_files {
            let content = std::fs::read_to_string(path)
                .map_err(|e| CompileError::IoError(path.clone(), e))?;
            sources.add(path.clone(), content);
        }

        Ok(Self {
            config,
            sources,
            diagnostics,
            def_id_gen,
        })
    }

    pub fn compile(&mut self) -> Result<CompilationResults, CompileError> {
        if self.config.verbose {
            eprintln!("[verbose] Starting compilation...");
            eprintln!("[verbose] Input files: {:?}", self.config.input_files);
        }

        if self.config.verbose {
            eprintln!("[verbose] Phase: Lexing & Parsing");
        }
        let mut all_tokens = Vec::new();
        let mut all_asts = Vec::new();

        for (file_id, source) in self.sources.iter() {
            if self.config.verbose {
                eprintln!("[verbose] Lexing: {}", source.path.display());
            }
            let mut lexer = Lexer::new(&source.content, &mut self.diagnostics);
            let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
                .take_while(|t| *t != faxc_lex::Token::Eof)
                .collect();

            if self.config.emit == EmitType::Tokens {
                all_tokens.push((file_id, tokens.clone()));
            }

            if self.config.verbose {
                eprintln!("[verbose] Parsing: {}", source.path.display());
            }
            let mut parser = Parser::new(tokens, &mut self.diagnostics);
            let ast = parser.parse();
            all_asts.push((file_id, ast));
        }

        if self.config.emit == EmitType::Ast {
            return Ok(CompilationResults {
                tokens: all_tokens,
                asts: all_asts,
                hirs: vec![],
                mirs: vec![],
                lirs: vec![],
                objects: vec![],
            });
        }

        if self.config.verbose {
            eprintln!("[verbose] Phase: Semantic Analysis");
        }
        let mut type_context = TypeContext::default();
        let mut all_hirs = Vec::new();
        for (file_id, ast) in &all_asts {
            if self.config.verbose {
                let source_name = self
                    .sources
                    .iter()
                    .find(|(fid, _)| *fid == *file_id)
                    .map(|(_, f)| f.path.display().to_string())
                    .unwrap_or_else(|| "<unknown>".to_string());
                eprintln!("[verbose] Analyzing: {}", source_name);
            }
            let mut analyzer =
                SemanticAnalyzer::new(&mut type_context, &self.def_id_gen, &mut self.diagnostics);
            let hir = analyzer.analyze_items(ast.clone());
            all_hirs.push((*file_id, hir));
        }

        if self.diagnostics.has_errors() {
            return Err(CompileError::CompilationFailed);
        }

        if self.config.emit == EmitType::Hir {
            return Ok(CompilationResults {
                tokens: vec![],
                asts: vec![],
                hirs: all_hirs,
                mirs: vec![],
                lirs: vec![],
                objects: vec![],
            });
        }

        let mut all_mirs = Vec::new();
        for (file_id, hir) in &all_hirs {
            for item in hir {
                if let HirItem::Function(func) = item {
                    let mir = lower_hir_function(func);
                    all_mirs.push((*file_id, mir));
                }
            }
        }

        if self.config.verbose {
            eprintln!("[verbose] Phase: MIR Optimization");
        }
        for (_, mir) in &mut all_mirs {
            optimize_mir(mir);
        }

        if self.config.emit == EmitType::Mir {
            return Ok(CompilationResults {
                tokens: vec![],
                asts: vec![],
                hirs: vec![],
                mirs: all_mirs,
                lirs: vec![],
                objects: vec![],
            });
        }

        let mut all_lirs = Vec::new();
        for (file_id, mir) in &all_mirs {
            let lir = lower_mir_to_lir(mir);
            all_lirs.push((*file_id, lir));
        }

        if self.config.verbose {
            eprintln!("[verbose] Phase: LIR Optimization");
        }
        for (_, lir) in &mut all_lirs {
            optimize_lir(lir);
        }

        if self.config.emit == EmitType::Lir {
            return Ok(CompilationResults {
                tokens: vec![],
                asts: vec![],
                hirs: vec![],
                mirs: vec![],
                lirs: all_lirs,
                objects: vec![],
            });
        }

        let context = inkwell::context::Context::create();
        let mut llvm_backend = LlvmBackend::new(
            &context,
            "fax_module",
            self.config.target.clone(),
            inkwell::OptimizationLevel::None,
        );

        for (_, lir) in &all_lirs {
            llvm_backend
                .compile_function(lir)
                .map_err(|e| CompileError::CodeGenError(e))?;
        }

        let llvm_ir = llvm_backend.emit_llvm_ir();
        let mut objects = Vec::new();
        objects.push((FileId(0), llvm_ir.clone()));

        let output_path = self.config.output_file.clone();

        if let Some(ref path) = output_path {
            match self.config.emit {
                EmitType::LlvmIr => {
                    std::fs::write(path, &llvm_ir)
                        .map_err(|e| CompileError::IoError(path.clone(), e))?;
                    if self.config.verbose {
                        eprintln!("[verbose] Wrote LLVM IR to {}", path.display());
                    }
                },
                EmitType::Asm => {
                    llvm_backend
                        .write_asm_file(path)
                        .map_err(|e| CompileError::CodeGenError(e))?;
                    if self.config.verbose {
                        eprintln!("[verbose] Wrote assembly to {}", path.display());
                    }
                },
                EmitType::Object => {
                    llvm_backend
                        .write_object_file(path)
                        .map_err(|e| CompileError::CodeGenError(e))?;
                    if self.config.verbose {
                        eprintln!("[verbose] Wrote object file to {}", path.display());
                    }
                },
                EmitType::Exe => {
                    let ir = llvm_backend.emit_llvm_ir();

                    let ir_file = std::env::temp_dir().join("fax_compile.ll");

                    // Inject GC initialization at the start of main
                    let ir_with_gc_init = inject_gc_init(&ir);
                    std::fs::write(&ir_file, &ir_with_gc_init)?;

                    // Find runtime library path - check multiple locations
                    let possible_paths = vec![
                        std::env::current_exe()
                            .ok()
                            .and_then(|p| p.parent().map(|p| p.join("libfaxc_runtime.so"))),
                        Some(std::path::PathBuf::from("target/debug/libfaxc_runtime.so")),
                        Some(std::path::PathBuf::from(
                            "faxc/target/debug/libfaxc_runtime.so",
                        )),
                        Some(std::path::PathBuf::from(
                            "/root/Fax/target/debug/libfaxc_runtime.so",
                        )),
                    ];

                    let runtime_lib = possible_paths.into_iter().flatten().find(|p| p.exists());

                    let mut cmd = std::process::Command::new("clang");
                    cmd.arg("-o").arg(path).arg("-x").arg("ir").arg(&ir_file);

                    // Link runtime if exists
                    if let Some(ref lib) = runtime_lib {
                        if self.config.verbose {
                            eprintln!("[verbose] Linking with FGC runtime: {}", lib.display());
                        }
                        // Use -L and -l for shared library
                        if lib.extension().map_or(false, |e| e == "so") {
                            if let Some(parent) = lib.parent() {
                                cmd.arg("-L").arg(parent);
                            }
                            cmd.arg("-lfaxc_runtime");
                            if let Some(parent) = lib.parent() {
                                cmd.arg(&format!("-Wl,-rpath,{}", parent.display()));
                            }
                        } else {
                            cmd.arg(lib);
                        }
                    } else {
                        if self.config.verbose {
                            eprintln!("[verbose] Runtime not found, using system malloc");
                        }
                    }

                    let result = cmd.output();

                    let _ = std::fs::remove_file(&ir_file);

                    match result {
                        Ok(output) if output.status.success() => {
                            if self.config.verbose {
                                eprintln!("[verbose] Linked executable to {}", path.display());
                            }
                        },
                        Ok(output) => {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            eprintln!("Warning: Linking failed: {}", stderr);
                            std::fs::write(path, &llvm_ir).ok();
                        },
                        Err(e) => {
                            eprintln!("Warning: Could not invoke clang: {}", e);
                            std::fs::write(path, &llvm_ir).ok();
                        },
                    }
                },
                _ => {},
            }
        }

        Ok(CompilationResults {
            tokens: vec![],
            asts: vec![],
            hirs: vec![],
            mirs: vec![],
            lirs: vec![],
            objects,
        })
    }
}

pub struct SourceMap {
    files: Vec<SourceFile>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self { files: Vec::new() }
    }
    pub fn add(&mut self, path: PathBuf, content: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile { path, content });
        id
    }
    pub fn iter(&self) -> impl Iterator<Item = (FileId, &SourceFile)> {
        self.files
            .iter()
            .enumerate()
            .map(|(i, f)| (FileId(i as u32), f))
    }
}

pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

pub struct CompilationResults {
    pub tokens: Vec<(FileId, Vec<faxc_lex::Token>)>,
    pub asts: Vec<(FileId, Vec<faxc_par::Item>)>,
    pub hirs: Vec<(FileId, Vec<faxc_sem::Item>)>,
    pub mirs: Vec<(FileId, faxc_mir::Function)>,
    pub lirs: Vec<(FileId, faxc_lir::Function)>,
    pub objects: Vec<(FileId, String)>,
}

#[derive(Debug)]
pub enum CompileError {
    IoError(PathBuf, std::io::Error),
    ParseError(String),
    NoInputFiles,
    CompilationFailed,
    CodeGenError(CodeGenError),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::IoError(p, e) => write!(f, "IO Error on {}: {}", p.display(), e),
            CompileError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CompileError::NoInputFiles => write!(f, "No input files provided"),
            CompileError::CompilationFailed => write!(f, "Compilation Failed"),
            CompileError::CodeGenError(e) => write!(f, "Code Generation Error: {}", e),
        }
    }
}

impl std::error::Error for CompileError {}

impl From<std::io::Error> for CompileError {
    fn from(err: std::io::Error) -> Self {
        CompileError::IoError(PathBuf::new(), err)
    }
}

pub fn main() -> Result<(), CompileError> {
    let config = parse_args().map_err(|e| CompileError::ParseError(e))?;

    if config.help {
        print_help();
        return Ok(());
    }

    if config.version {
        print_version();
        return Ok(());
    }

    if config.input_files.is_empty() {
        return Err(CompileError::NoInputFiles);
    }

    let mut session = Session::new(config)?;
    session.compile()?;
    Ok(())
}

fn default_target() -> String {
    let config = inkwell::targets::InitializationConfig::default();
    inkwell::targets::Target::initialize_native(&config).ok();
    let triple = inkwell::targets::TargetMachine::get_default_triple();
    triple.as_str().to_string_lossy().into_owned()
}

fn inject_gc_init(ir: &str) -> String {
    if ir.contains("define i64 @main(") {
        let gc_init = "; Fax GC initialization\n\
define void @__fax_gc_init() {\n\
entry:\n\
    %0 = call i1 @fax_gc_init()\n\
    ret void\n\
}\n\
\n";
        let gc_init_call = "; Call GC initialization\n\
    call void @__fax_gc_init()\n\
";
        let main_pattern = "define i64 @main(";
        if let Some(pos) = ir.find(main_pattern) {
            let before = &ir[..pos];
            let after = &ir[pos..];
            let with_call = after.replacen("entry:", &format!("entry:\n{}", gc_init_call), 1);
            return format!("{}{}{}", before, gc_init, with_call);
        }
    }
    ir.to_string()
}
