//! faxc-drv - Compiler Driver
//!
//! ============================================================================
//! COMPILER DRIVER OVERVIEW
//! ============================================================================
//!
//! Compiler driver adalah entry point dan orchestrator untuk seluruh
//! compilation pipeline. Driver bertanggung jawab untuk:
//!
//! 1. COMMAND LINE PARSING
//!    - Parse arguments dan flags
//!    - Validate options
//!    - Setup configuration
//!
//! 2. FILE MANAGEMENT
//!    - Read source files
//!    - Manage output paths
//!
//! 3. PIPELINE ORCHESTRATION
//!    - Run compilation phases dalam urutan yang benar
//!    - Handle errors antar phases
//!    - Manage intermediate artifacts
//!
//! 4. ERROR REPORTING
//!    - Aggregate diagnostics dari semua phases
//!    - Format dan display errors
//!    - Exit dengan appropriate code
//!
//! ============================================================================
//! COMPILATION PIPELINE
//! ============================================================================
//!
//! ```
//! Source Files (.fax)
//!        │
//!        ▼
//!   [Read Files]
//!        │
//!        ▼
//!   [Lexer] ──▶ Token Stream
//!        │
//!        ▼
//!   [Parser] ──▶ AST
//!        │
//!        ▼
//!   [Semantic Analysis] ──▶ HIR
//!        │
//!        ▼
//!   [MIR Generation] ──▶ MIR
//!        │
//!        ▼
//!   [MIR Optimization] ──▶ Optimized MIR
//!        │
//!        ▼
//!   [LIR Generation] ──▶ LIR
//!        │
//!        ▼
//!   [Register Allocation] ──▶ LIR with Physical Registers
//!        │
//!        ▼
//!   [Code Generation] ──▶ Assembly / Object File
//!        │
//!        ▼
//!   [Linking] ──▶ Executable
//! ```
//!
//! PHASES DETAIL:
//! --------------
//!
//! Phase 1: Lexical Analysis
//! - Input: Source code (text)
//! - Output: Token stream
//! - Tool: faxc-lex
//! - Errors: Invalid characters, unterminated strings
//!
//! Phase 2: Parsing
//! - Input: Token stream
//! - Output: Abstract Syntax Tree (AST)
//! - Tool: faxc-par
//! - Errors: Syntax errors, unexpected tokens
//!
//! Phase 3: Semantic Analysis
//! - Input: AST
//! - Output: High-level IR (HIR)
//! - Tool: faxc-sem
//! - Errors: Type errors, undefined names, borrow check errors
//!
//! Phase 4: MIR Generation
//! - Input: HIR
//! - Output: Mid-level IR (MIR)
//! - Tool: faxc-mir
//! - Tasks: Lower ke SSA form, build CFG
//! - Special: Transform async/await ke state machine
//!
//! Phase 5: MIR Optimization
//! - Input: MIR
//! - Output: Optimized MIR
//! - Tasks: Constant folding, DCE, inlining, etc.
//!
//! Phase 6: LIR Generation
//! - Input: Optimized MIR
//! - Output: Low-level IR (LIR)
//! - Tasks: Lower PHI nodes, explicit memory ops
//!
//! Phase 7: Register Allocation
//! - Input: LIR with virtual registers
//! - Output: LIR with physical registers
//! - Tasks: Graph coloring, spilling
//!
//! Phase 8: Code Generation
//! - Input: LIR
//! - Output: Assembly or object file
//! - Tool: faxc-gen
//!
//! Phase 9: Linking
//! - Input: Object files
//! - Output: Executable
//! - Tool: System linker (ld, link.exe, etc.)
//!
//! ============================================================================
//! COMMAND LINE INTERFACE
//! ============================================================================
//!
//! USAGE:
//!
//! Compile file:
//!   faxc main.fax
//!
//! Specify output:
//!   faxc main.fax -o myprogram
//!
//! Multiple files:
//!   faxc file1.fax file2.fax file3.fax -o program
//!
//! Optimization levels:
//!   faxc -O0 main.fax    # No optimization (debug)
//!   faxc -O1 main.fax    # Basic optimization
//!   faxc -O2 main.fax    # Standard optimization (default)
//!   faxc -O3 main.fax    # Aggressive optimization
//!   faxc -Os main.fax    # Optimize for size
//!
//! Emit intermediate representations:
//!   faxc --emit-tokens main.fax     # Lexer output
//!   faxc --emit-ast main.fax        # Parser output
//!   faxc --emit-hir main.fax        # HIR
//!   faxc --emit-mir main.fax        # MIR
//!   faxc --emit-lir main.fax        # LIR
//!   faxc --emit-asm main.fax        # Assembly
//!   faxc -S main.fax                # Same as --emit-asm
//!
//! Stop after specific phase:
//!   faxc -c main.fax      # Compile to object file only
//!
//! Cross compilation:
//!   faxc --target x86_64-pc-windows-gnu main.fax
//!   faxc --target aarch64-unknown-linux-gnu main.fax
//!
//! Debug options:
//!   faxc -g main.fax              # Include debug info
//!   faxc --verbose main.fax       # Verbose output
//!   faxc -Werror main.fax         # Treat warnings as errors
//!
//! Async/await support:
//!   faxc supports async/await for asynchronous programming.
//!   Example source:
//!     async fn fetch_data() -> string {
//!       let response = await http_get("https://example.com");
//!       response.body
//!     }
//!
//!     async fn main() {
//!       let data = await fetch_data();
//!       println(data);
//!     }
//!
//! ============================================================================
//! CONFIGURATION
//! ============================================================================
//!
//! Configuration mencakup semua options yang mempengaruhi compilation.
//!
//! Fields:
//! - input_files: Vec<PathBuf>
//! - output_file: Option<PathBuf>
//! - optimization_level: OptLevel
//! - target: String
//! - emit: EmitType
//! - debug: bool
//! - verbose: bool
//! - warnings_as_errors: bool
//! - libraries: Vec<String>
//! - library_paths: Vec<PathBuf>
//!
//! ============================================================================
//! ERROR HANDLING
//! ============================================================================
//!
//! Error Levels:
//! -------------
//! - ERROR: Fatal error, compilation fails
//! - WARNING: Non-fatal, compilation succeeds
//! - NOTE: Additional information
//! - HELP: Suggestion for fix
//!
//! Error Aggregation:
//! ------------------
//! Compiler mengumpulkan semua error sebelum exit.
//! Ini memungkinkan user untuk melihat semua masalah dalam sekali compile.
//!
//! Error Recovery:
//! ---------------
//! Setiap phase mencoba recover dari errors untuk melanjutkan
//! compilation dan menemukan lebih banyak errors.
//!
//! Exit Codes:
//! -----------
//! - 0: Success
//! - 1: Compilation error
//! - 2: Internal compiler error
//! - 3: Command line error
//!
//! ============================================================================
//! INCREMENTAL COMPILATION
//! ============================================================================
//!
//! Incremental compilation hanya recompile file yang berubah atau
//! depend on changed files.
//!
//! IMPLEMENTATION:
//! ---------------
//!
//! 1. Dependency Tracking
//!    - Hash setiap source file
//!    - Track module dependencies
//!    - Save dependency graph
//!
//! 2. Caching
//!    - Cache intermediate results (AST, HIR, MIR)
//!    - Save ke disk dalam format binary
//!
//! 3. Invalidation
//!    - Jika file berubah (hash berbeda), invalidate cache
//!    - Invalidate semua dependent files
//!
//! 4. Partial Recompilation
//!    - Compile hanya file yang invalidated
//!    - Link dengan object files yang sudah ada

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Compiler configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Input source files
    pub input_files: Vec<PathBuf>,

    /// Output file path (None untuk default)
    pub output_file: Option<PathBuf>,

    /// Optimization level
    pub opt_level: OptLevel,

    /// Target triple
    pub target: String,

    /// Emit type (what to produce)
    pub emit: EmitType,

    /// Include debug information
    pub debug: bool,

    /// Verbose output
    pub verbose: bool,

    /// Treat warnings as errors
    pub warnings_as_errors: bool,

    /// Libraries to link
    pub libraries: Vec<String>,

    /// Library search paths
    pub library_paths: Vec<PathBuf>,

    /// Enable incremental compilation
    pub incremental: bool,

    /// Working directory
    pub working_dir: PathBuf,
}

/// Optimization level
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OptLevel {
    /// No optimization
    None,
    /// Basic optimization
    Less,
    /// Standard optimization
    Default,
    /// Aggressive optimization
    Aggressive,
    /// Optimize for size
    Size,
}

impl Default for OptLevel {
    fn default() -> Self {
        OptLevel::Default
    }
}

/// Emit type - what output to produce
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitType {
    /// Tokens only
    Tokens,
    /// AST only
    Ast,
    /// HIR only
    Hir,
    /// MIR only
    Mir,
    /// LIR only
    Lir,
    /// Assembly
    Asm,
    /// Object file
    Object,
    /// Full executable
    Executable,
}

impl Default for EmitType {
    fn default() -> Self {
        EmitType::Executable
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            input_files: Vec::new(),
            output_file: None,
            opt_level: OptLevel::Default,
            target: default_target(),
            emit: EmitType::Executable,
            debug: false,
            verbose: false,
            warnings_as_errors: false,
            libraries: Vec::new(),
            library_paths: Vec::new(),
            incremental: true,
            working_dir: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

/// Compilation session
///
/// Session menyimpan state untuk satu invocation compiler.
pub struct Session {
    /// Configuration
    pub config: Config,

    /// Source map (all loaded files)
    pub sources: SourceMap,

    /// Diagnostic handler
    pub diagnostics: DiagnosticHandler,

    /// String interner
    pub interner: Interner,

    /// Incremental cache
    pub cache: Option<IncrementalCache>,
}

impl Session {
    /// Create new session
    pub fn new(config: Config) -> Self {
        let cache = if config.incremental {
            Some(IncrementalCache::new())
        } else {
            None
        };

        Self {
            config,
            sources: SourceMap::new(),
            diagnostics: DiagnosticHandler::new(),
            interner: Interner::new(),
            cache,
        }
    }

    /// Run compilation
    pub fn compile(&mut self) -> Result<(), CompileError> {
        if self.config.verbose {
            eprintln!("Configuration: {:?}", self.config);
        }

        // Phase 1: Read source files
        self.read_sources()?;

        // Phase 2-8: Run compiler pipeline
        let results = self.run_pipeline()?;

        // Phase 9: Output
        self.emit_output(results)?;

        // Check for errors
        if self.diagnostics.has_errors() {
            return Err(CompileError::CompilationFailed);
        }

        Ok(())
    }

    /// Read all source files
    fn read_sources(&mut self) -> Result<(), CompileError> {
        for path in &self.config.input_files {
            if self.config.verbose {
                eprintln!("Reading: {}", path.display());
            }

            let content = std::fs::read_to_string(path)
                .map_err(|e| CompileError::IoError(path.clone(), e))?;

            self.sources.add_file(path.clone(), content);
        }

        Ok(())
    }

    /// Run compilation pipeline
    fn run_pipeline(&mut self) -> Result<CompilationResults, CompileError> {
        use faxc_lex::Lexer;
        use faxc_lir::lower_mir_to_lir;
        use faxc_mir::lower_hir_function;
        use faxc_par::Parser;
        use faxc_sem::{SemanticAnalyzer, TypeContext};

        let mut all_tokens = Vec::new();
        let mut all_asts = Vec::new();
        let mut all_hirs = Vec::new();
        let mut all_mirs = Vec::new();
        let mut all_lirs = Vec::new();

        for (file_id, source) in self.sources.iter() {
            // Phase 2: Lexing
            if self.config.verbose {
                eprintln!("Lexing file: {:?}", file_id);
            }

            let mut lexer = Lexer::new(&source.content, &mut self.diagnostics);
            let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
                .take_while(|t| *t != faxc_lex::Token::Eof)
                .collect();

            if self.config.emit == EmitType::Tokens {
                all_tokens.push((file_id, tokens));
                continue;
            }

            // Phase 3: Parsing
            if self.config.verbose {
                eprintln!("Parsing file: {:?}", file_id);
            }

            let mut parser = Parser::new(tokens, &mut self.diagnostics);
            let ast = parser.parse();

            if self.config.emit == EmitType::Ast {
                all_asts.push((file_id, ast));
                continue;
            }

            all_asts.push((file_id, ast));
        }

        // Stop here jika hanya emit tokens atau AST
        if matches!(self.config.emit, EmitType::Tokens | EmitType::Ast) {
            return Ok(CompilationResults {
                tokens: all_tokens,
                asts: all_asts,
                hirs: Vec::new(),
                mirs: Vec::new(),
                lirs: Vec::new(),
                objects: Vec::new(),
            });
        }

        // Phase 4: Semantic Analysis
        if self.config.verbose {
            eprintln!("Semantic analysis...");
        }

        let mut type_context = TypeContext::new();

        for (file_id, ast) in &all_asts {
            let mut analyzer = SemanticAnalyzer::new(&mut type_context, &mut self.diagnostics);
            let hir = analyzer.analyze(ast.clone());
            all_hirs.push((*file_id, hir));
        }

        if self.config.emit == EmitType::Hir {
            return Ok(CompilationResults {
                tokens: Vec::new(),
                asts: Vec::new(),
                hirs: all_hirs,
                mirs: Vec::new(),
                lirs: Vec::new(),
                objects: Vec::new(),
            });
        }

        // Phase 5: MIR Generation
        if self.config.verbose {
            eprintln!("MIR generation...");
        }

        for (file_id, hir) in &all_hirs {
            for item in hir {
                if let faxc_sem::Item::Function(func) = item {
                    let mir = lower_hir_function(func);
                    all_mirs.push((*file_id, mir));
                }
            }
        }

        if self.config.emit == EmitType::Mir {
            return Ok(CompilationResults {
                tokens: Vec::new(),
                asts: Vec::new(),
                hirs: Vec::new(),
                mirs: all_mirs,
                lirs: Vec::new(),
                objects: Vec::new(),
            });
        }

        // Phase 6: MIR Optimization
        if self.config.verbose {
            eprintln!("MIR optimization...");
        }

        // TODO: Run optimization passes

        // Phase 7: LIR Generation
        if self.config.verbose {
            eprintln!("LIR generation...");
        }

        for (file_id, mir) in &all_mirs {
            let lir = lower_mir_to_lir(mir);
            all_lirs.push((*file_id, lir));
        }

        if self.config.emit == EmitType::Lir {
            return Ok(CompilationResults {
                tokens: Vec::new(),
                asts: Vec::new(),
                hirs: Vec::new(),
                mirs: Vec::new(),
                lirs: all_lirs,
                objects: Vec::new(),
            });
        }

        // Phase 8: Code Generation
        if self.config.verbose {
            eprintln!("Code generation...");
        }

        let mut objects = Vec::new();

        for (file_id, lir) in &all_lirs {
            use faxc_gen::AsmGenerator;

            let mut gen = AsmGenerator::new();
            gen.generate_function(lir);

            objects.push((*file_id, gen.output().to_string()));
        }

        Ok(CompilationResults {
            tokens: Vec::new(),
            asts: Vec::new(),
            hirs: Vec::new(),
            mirs: Vec::new(),
            lirs: Vec::new(),
            objects,
        })
    }

    /// Emit output
    fn emit_output(&self, results: CompilationResults) -> Result<(), CompileError> {
        match self.config.emit {
            EmitType::Tokens => {
                for (_, tokens) in results.tokens {
                    println!("{:?}", tokens);
                }
            }
            EmitType::Ast => {
                for (_, ast) in results.asts {
                    println!("{:#?}", ast);
                }
            }
            EmitType::Hir => {
                for (_, hir) in results.hirs {
                    println!("{:#?}", hir);
                }
            }
            EmitType::Mir => {
                for (_, mir) in results.mirs {
                    println!("{:#?}", mir);
                }
            }
            EmitType::Lir => {
                for (_, lir) in results.lirs {
                    println!("{:#?}", lir);
                }
            }
            EmitType::Asm | EmitType::Object | EmitType::Executable => {
                // Write assembly atau compile kemudian link
                let output = self
                    .config
                    .output_file
                    .as_ref()
                    .map(|p| p.as_path())
                    .unwrap_or(Path::new("a.out"));

                if results.objects.len() == 1 {
                    std::fs::write(output, &results.objects[0].1)
                        .map_err(|e| CompileError::IoError(output.to_path_buf(), e))?;
                }
            }
        }

        Ok(())
    }
}

/// Compilation results
pub struct CompilationResults {
    pub tokens: Vec<(FileId, Vec<faxc_lex::Token>)>,
    pub asts: Vec<(FileId, faxc_par::Ast)>,
    pub hirs: Vec<(FileId, Vec<faxc_sem::Item>)>,
    pub mirs: Vec<(FileId, faxc_mir::Function)>,
    pub lirs: Vec<(FileId, faxc_lir::Function)>,
    pub objects: Vec<(FileId, String)>,
}

/// File ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// Source map
pub struct SourceMap {
    files: HashMap<FileId, SourceFile>,
    next_id: u32,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn add_file(&mut self, path: PathBuf, content: String) -> FileId {
        let id = FileId(self.next_id);
        self.next_id += 1;

        self.files.insert(id, SourceFile { path, content });
        id
    }

    pub fn get(&self, id: FileId) -> Option<&SourceFile> {
        self.files.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = (FileId, &SourceFile)> {
        self.files.iter().map(|(k, v)| (*k, v))
    }
}

/// Source file
pub struct SourceFile {
    pub path: PathBuf,
    pub content: String,
}

/// Diagnostic handler
pub struct DiagnosticHandler {
    errors: Vec<Diagnostic>,
    warnings: Vec<Diagnostic>,
}

impl DiagnosticHandler {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn emit(&mut self, diag: Diagnostic) {
        match diag.level {
            Level::Error => self.errors.push(diag),
            Level::Warning => self.warnings.push(diag),
            _ => {}
        }
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

/// Diagnostic
pub struct Diagnostic {
    pub level: Level,
    pub message: String,
    pub span: Span,
}

/// Diagnostic level
pub enum Level {
    Error,
    Warning,
    Note,
    Help,
}

/// Span
pub struct Span;

impl Span {
    pub const DUMMY: Span = Span;
}

/// String interner
pub struct Interner;

impl Interner {
    pub fn new() -> Self {
        Self
    }
}

/// Incremental cache
pub struct IncrementalCache;

impl IncrementalCache {
    pub fn new() -> Self {
        Self
    }
}

/// Compile error
#[derive(Debug)]
pub enum CompileError {
    IoError(PathBuf, std::io::Error),
    CompilationFailed,
    InvalidArguments(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::IoError(path, e) => {
                write!(f, "IO error for {}: {}", path.display(), e)
            }
            CompileError::CompilationFailed => {
                write!(f, "Compilation failed")
            }
            CompileError::InvalidArguments(s) => {
                write!(f, "Invalid arguments: {}", s)
            }
        }
    }
}

impl std::error::Error for CompileError {}

/// Get default target triple
fn default_target() -> String {
    std::env::var("TARGET").unwrap_or_else(|_| {
        if cfg!(target_os = "linux") {
            "x86_64-unknown-linux-gnu".to_string()
        } else if cfg!(target_os = "macos") {
            "x86_64-apple-darwin".to_string()
        } else if cfg!(target_os = "windows") {
            "x86_64-pc-windows-msvc".to_string()
        } else {
            "x86_64-unknown-unknown".to_string()
        }
    })
}
