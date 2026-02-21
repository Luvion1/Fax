//! faxc-drv - Compiler Driver
//!
//! Driver utama yang mengkoordinasikan seluruh tahapan kompilasi.

use std::path::PathBuf;
use faxc_lex::Lexer;
use faxc_par::Parser;
use faxc_sem::{SemanticAnalyzer, TypeContext, Item as HirItem};
use faxc_mir::lower_hir_function;
use faxc_lir::{lower_mir_to_lir, Function as LirFunction};
use faxc_gen::LlvmBackend;
use faxc_util::{Handler, Symbol, DefIdGenerator};

/// Configuration untuk compiler
#[derive(Debug, Clone)]
pub struct Config {
    pub input_files: Vec<PathBuf>,
    pub output_file: Option<PathBuf>,
    pub target: String,
    pub emit: EmitType,
    pub verbose: bool,
    pub incremental: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmitType {
    Tokens,
    Ast,
    Hir,
    Mir,
    Lir,
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
        }
    }
}

/// Session kompilasi
pub struct Session {
    pub config: Config,
    pub sources: SourceMap,
    pub diagnostics: Handler,
    pub def_id_gen: DefIdGenerator,
}

impl Session {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            sources: SourceMap::new(),
            diagnostics: Handler::new(),
            def_id_gen: DefIdGenerator::new(),
        }
    }

    pub fn compile(&mut self) -> Result<CompilationResults, CompileError> {
        // 2. Lexing & Parsing
        let mut all_tokens = Vec::new();
        let mut all_asts = Vec::new();
        
        for (file_id, source) in self.sources.iter() {
            let mut lexer = Lexer::new(&source.content, &mut self.diagnostics);
            let tokens: Vec<_> = std::iter::from_fn(|| Some(lexer.next_token()))
                .take_while(|t| *t != faxc_lex::Token::Eof)
                .collect();
            
            if self.config.emit == EmitType::Tokens {
                all_tokens.push((file_id, tokens.clone()));
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

        // 3. Semantic Analysis
        let mut type_context = TypeContext::default();
        let mut all_hirs = Vec::new();
        for (file_id, ast) in &all_asts {
            let mut analyzer = SemanticAnalyzer::new(&mut type_context, &self.def_id_gen, &mut self.diagnostics);
            let hir = analyzer.analyze_items(ast.clone());
            all_hirs.push((*file_id, hir));
        }

        if self.config.emit == EmitType::Hir {
            return Ok(CompilationResults {
                tokens: vec![], asts: vec![], hirs: all_hirs, mirs: vec![], lirs: vec![], objects: vec![]
            });
        }

        // 4. MIR Generation
        let mut all_mirs = Vec::new();
        for (file_id, hir) in &all_hirs {
            for item in hir {
                if let HirItem::Function(func) = item {
                    let mir = lower_hir_function(func);
                    all_mirs.push((*file_id, mir));
                }
            }
        }

        if self.config.emit == EmitType::Mir {
            return Ok(CompilationResults {
                tokens: vec![], asts: vec![], hirs: vec![], mirs: all_mirs, lirs: vec![], objects: vec![]
            });
        }

        // 5. LIR Generation
        let mut all_lirs = Vec::new();
        for (file_id, mir) in &all_mirs {
            let lir = lower_mir_to_lir(mir);
            all_lirs.push((*file_id, lir));
        }

        if self.config.emit == EmitType::Lir {
            return Ok(CompilationResults {
                tokens: vec![], asts: vec![], hirs: vec![], mirs: vec![], lirs: all_lirs, objects: vec![]
            });
        }

        // 6. LLVM Code Generation
        let context = inkwell::context::Context::create();
        let mut llvm_backend = LlvmBackend::new(
            &context,
            "fax_module",
            self.config.target.clone(),
            inkwell::OptimizationLevel::None,
        );

        for (_, lir) in &all_lirs {
            llvm_backend.compile_function(lir);
        }

        let llvm_ir = llvm_backend.emit_llvm_ir();
        let mut objects = Vec::new();
        objects.push((FileId(0), llvm_ir));

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
    pub fn new() -> Self { Self { files: Vec::new() } }
    pub fn add(&mut self, path: PathBuf, content: String) -> FileId {
        let id = FileId(self.files.len() as u32);
        self.files.push(SourceFile { path, content });
        id
    }
    pub fn iter(&self) -> impl Iterator<Item = (FileId, &SourceFile)> {
        self.files.iter().enumerate().map(|(i, f)| (FileId(i as u32), f))
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
    CompilationFailed,
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::IoError(p, e) => write!(f, "IO Error on {}: {}", p.display(), e),
            CompileError::CompilationFailed => write!(f, "Compilation Failed"),
        }
    }
}

impl std::error::Error for CompileError {}

pub fn main() -> Result<(), CompileError> {
    let config = Config::default();
    let mut session = Session::new(config);
    session.compile()?;
    Ok(())
}

fn default_target() -> String {
    "x86_64-unknown-linux-gnu".to_string()
}
