//! faxc-gen - Code Generator
//!
//! ============================================================================
//! CODE GENERATION OVERVIEW
//! ============================================================================
//!
//! Code generation adalah fase terakhir compiler yang mengubah intermediate
//! representation (LIR) menjadi executable machine code.
//!
//! PROSES CODE GENERATION:
//! -----------------------
//!
//! 1. INSTRUCTION SELECTION
//!    Memilih instruksi machine yang sesuai untuk setiap LIR instruction.
//!
//! 2. REGISTER ALLOCATION
//!    Memetakan virtual registers ke physical registers.
//!
//! 3. INSTRUCTION SCHEDULING (optional)
//!    Mengoptimasi urutan instruksi untuk pipeline efficiency.
//!
//! 4. ASSEMBLY GENERATION
//!    Menghasilkan assembly language text (jika diminta).
//!
//! 5. OBJECT FILE GENERATION
//!    Menghasilkan binary object file (ELF, Mach-O, COFF).
//!
//! 6. LINKING
//!    Menggabungkan object files menjadi executable.
//!
//! ============================================================================
//! BACKEND ARCHITECTURES
//! ============================================================================
//!
//! FGC menggunakan dua pendekatan backend:
//!
//! 1. LLVM BACKEND (Primary)
//! -------------------------
//! - Generate LLVM IR dari LIR
//! - Gunakan LLVM optimizer dan code generator
//! - Support multiple targets (x86, ARM, WASM, etc.)
//! - Mature dan well-tested
//!
//! Keuntungan:
//! - Portabilitas tinggi (support banyak architecture)
//! - Optimasi yang sangat baik
//! - Maintenance lebih mudah
//!
//! Kekurangan:
//! - Compile time lebih lambat
//! - Dependency besar
//!
//! 2. DIRECT ASSEMBLY (Optional)
//! -----------------------------
//! - Generate assembly language langsung
//! - Full control atas generated code
//! - Faster compile times (tanpa LLVM overhead)
//!
//! Keuntungan:
//! - Full control
//! - Faster compilation
//! - No external dependency
//!
//! Kekurangan:
//! - Platform-specific
//! - More code to maintain
//!
//! ============================================================================
//! REGISTER ALLOCATION
//! ============================================================================
//!
//! Register allocation adalah proses memetakan virtual registers dari LIR
//! ke physical registers yang terbatas.
//!
//! GRAPH COLORING ALGORITHM:
//! -------------------------
//!
//! 1. BUILD INTERFERENCE GRAPH
//!    - Nodes = virtual registers
//!    - Edge = two registers yang live pada saat yang bersamaan
//!
//! 2. COLOR THE GRAPH
//!    Coba warnai graph dengan N warna (N = jumlah physical registers)
//!    - Jika bisa: assign register
//!    - Jika tidak: spill ke memory (stack)
//!
//! 3. HANDLE SPILLS
//!    Insert load/store instructions untuk akses spilled values
//!
//! CONTOH:
//!
//! LIR:
//! ```
//! v0 = 5
//! v1 = v0 + 10
//! v2 = v1 * 2
//! ```
//!
//! Interference:
//! - v0 dan v1 interfere (v0 live saat v1 dibuat)
//! - v1 dan v2 interfere
//! - v0 dan v2 tidak interfere
//!
//! Coloring (2 registers: RAX, RBX):
//! - v0 = RAX
//! - v1 = RBX
//! - v2 = RAX (reuse karena v0 sudah mati)
//!
//! SPILLING:
//! ---------
//! Jika tidak cukup registers, beberapa values harus disimpan di stack.
//!
//! LIR:
//! ```
//! v0 = ...
//! ... (banyak operasi, semua registers terpakai)
//! use(v0)  // v0 masih diperlukan tapi tidak ada register kosong
//! ```
//!
//! Solusi:
//! ```
//! v0 = ...
//! [spill] store [rbp-8], v0  // Save ke stack
//! ... (banyak operasi)
//! [reload] v0 = load [rbp-8] // Restore dari stack
//! use(v0)
//! ```
//!
//! x86-64 REGISTER CONVENTIONS:
//! ----------------------------
//!
//! General Purpose Registers:
//! - RAX, RBX, RCX, RDX, RSI, RDI, RBP, RSP
//! - R8-R15
//!
//! Special Purpose:
//! - RAX: Return value, accumulator
//! - RCX: Loop counter
//! - RDX: Extended accumulator
//! - RBX: Base register (callee-saved)
//! - RSP: Stack pointer
//! - RBP: Base pointer (callee-saved)
//! - RSI, RDI: Source/Destination index
//!
//! Calling Convention (System V AMD64 ABI):
//! ----------------------------------------
//! Arguments passing:
//! - RDI, RSI, RDX, RCX, R8, R9 (first 6 integer/pointer args)
//! - XMM0-XMM7 (first 8 floating point args)
//! - Stack (additional arguments)
//!
//! Return values:
//! - RAX (integer/pointer)
//! - RDX:RAX (128-bit)
//! - XMM0 (float)
//!
//! Callee-saved (harus dipreserve):
//! - RBX, RBP, R12, R13, R14, R15
//!
//! Caller-saved (bisa diubah):
//! - RAX, RCX, RDX, RSI, RDI, R8-R11
//!
//! ============================================================================
//! ASSEMBLY GENERATION
//! ============================================================================
//!
//! Format assembly x86-64 (AT&T syntax):
//!
//! ```asm
//!     .globl main
//!     .text
//! main:
//!     push rbp
//!     mov rbp, rsp
//!     sub rsp, 16           # Allocate 16 bytes for locals
//!
//!     # Function body
//!     mov rax, 42
//!
//!     mov rsp, rbp          # Deallocate locals
//!     pop rbp
//!     ret
//! ```
//!
//! STACK FRAME LAYOUT:
//! -------------------
//!
//! ```
//! Higher addresses
//!     │
//!     │  Return address
//!     ├───────────────────  <- RBP (base pointer)
//!     │
//!     │  Saved registers
//!     │
//!     │  Local variables
//!     │
//!     ├───────────────────  <- RSP (stack pointer)
//!     │
//!     ▼
//! Lower addresses
//! ```
//!
//! ============================================================================
//! OBJECT FILE FORMATS
//! ============================================================================
//!
//! ELF (Executable and Linkable Format) - Linux:
//! ----------------------------------------------
//!
//! Structure:
//! - ELF Header
//! - Program Header Table (for executables)
//! - Section Header Table
//! - Sections:
//!   * .text - Executable code
//!   * .data - Initialized global data
//!   * .bss - Uninitialized global data
//!   * .rodata - Read-only data
//!   * .symtab - Symbol table
//!   * .strtab - String table
//!   * .rel.text - Relocations for .text
//!
//! Mach-O - macOS:
//! ---------------
//! Structure mirip ELF tapi dengan beberapa perbedaan dalam header dan
//! load commands.
//!
//! PE/COFF - Windows:
//! ------------------
//! Format yang berbeda, menggunakan section table dan import/export tables.
//!
//! ============================================================================
//! LINKING
//! ============================================================================
//!
//! Linker menggabungkan multiple object files menjadi satu executable.
//!
//! TASKS:
//! ------
//! 1. Symbol Resolution
//!    Menghubungkan references ke definitions di object files berbeda.
//!
//! 2. Relocation
//!    Mengupdate addresses dengan lokasi final di executable.
//!
//! 3. Section Merging
//!    Menggabungkan sections yang sama dari object files berbeda.
//!
//! 4. Library Linking
//!    Menghubungkan dengan static libraries (.a/.lib) atau
//!    dynamic libraries (.so/.dll/.dylib).
//!
//! STATIC VS DYNAMIC LINKING:
//! --------------------------
//! Static: Library code dicopy ke executable
//! - Pros: Self-contained, faster startup
//! - Cons: Larger executable, update requires recompile
//!
//! Dynamic: Library di-load saat runtime
//! - Pros: Smaller executable, shared library updates
//! - Cons: Dependency management, slower startup

use faxc_lir::{Function as LirFunction, Instruction, Value, Register, BinOp, UnOp, Condition};
use faxc_util::Symbol;
use std::path::Path;
use std::collections::HashMap;

/// LLVM Backend untuk code generation
pub struct LlvmBackend {
    /// LLVM context
    context: (), // LLVMContextRef
    
    /// LLVM module
    module: (), // LLVMModuleRef
    
    /// LLVM builder
    builder: (), // LLVMBuilderRef
    
    /// Target triple
    target_triple: String,
    
    /// Optimization level
    opt_level: OptLevel,
}

/// Optimization level
#[derive(Debug, Clone, Copy)]
pub enum OptLevel {
    None,       // -O0
    Less,       // -O1
    Default,    // -O2
    Aggressive, // -O3
    Size,       // -Os
    SizeAggressive, // -Oz
}

impl LlvmBackend {
    /// Create new LLVM backend
    pub fn new(target_triple: String, opt_level: OptLevel) -> Self {
        Self {
            context: (),
            module: (),
            builder: (),
            target_triple,
            opt_level,
        }
    }
    
    /// Compile LIR function ke LLVM
    pub fn compile_function(&mut self, func: &LirFunction) {
        // 1. Create LLVM function type
        // 2. Create LLVM function
        // 3. Lower LIR instructions ke LLVM IR
        unimplemented!("LLVM compilation not implemented")
    }
    
    /// Optimize module
    pub fn optimize(&mut self) {
        // Run LLVM optimization passes
        unimplemented!("LLVM optimization not implemented")
    }
    
    /// Emit object file
    pub fn emit_object(&self, path: &Path) -> Result<(), CodegenError> {
        unimplemented!("Object emission not implemented")
    }
    
    /// Emit assembly file
    pub fn emit_asm(&self, path: &Path) -> Result<(), CodegenError> {
        unimplemented!("Assembly emission not implemented")
    }
}

/// Direct assembly generator
pub struct AsmGenerator {
    /// Output buffer
    output: String,
    
    /// Current indentation level
    indent: usize,
    
    /// Label counter
    label_counter: u32,
    
    /// Register allocator
    reg_alloc: RegisterAllocator,
}

/// Register allocator
pub struct RegisterAllocator {
    /// Mapping dari virtual register ke physical register atau stack slot
    allocation: HashMap<Register, Location>,
    
    /// Stack frame size
    frame_size: u32,
}

/// Allocation location
#[derive(Debug, Clone)]
pub enum Location {
    /// Physical register
    PhysReg(String),  // "rax", "rbx", etc.
    
    /// Stack slot
    Stack(i32),       // Offset dari RBP
}

impl AsmGenerator {
    /// Create new assembly generator
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            label_counter: 0,
            reg_alloc: RegisterAllocator {
                allocation: HashMap::new(),
                frame_size: 0,
            },
        }
    }
    
    /// Generate assembly untuk function
    pub fn generate_function(&mut self, func: &LirFunction) {
        // Emit function prologue
        self.emit_line(&format!(".globl {}", func.name));
        self.emit_line(&format!(".text"));
        self.emit_label(&func.name.to_string());
        
        // Standard function prologue
        self.emit_line("push rbp");
        self.emit_line("mov rbp, rsp");
        
        if self.reg_alloc.frame_size > 0 {
            self.emit_line(&format!("sub rsp, {}", self.reg_alloc.frame_size));
        }
        
        // Allocate registers
        self.allocate_registers(func);
        
        // Generate instructions
        for insn in &func.instructions {
            self.generate_instruction(insn);
        }
        
        // Standard function epilogue (jika belum ada return)
        self.emit_line("mov rsp, rbp");
        self.emit_line("pop rbp");
        self.emit_line("ret");
    }
    
    /// Allocate registers
    fn allocate_registers(&mut self, func: &LirFunction) {
        // Simple allocator: assign sequential physical registers
        // Real implementation would use graph coloring
        
        let phys_regs = ["rax", "rbx", "rcx", "rdx", "rsi", "rdi", "r8", "r9",
                        "r10", "r11", "r12", "r13", "r14", "r15"];
        
        for (i, reg) in func.registers.iter().enumerate() {
            if i < phys_regs.len() {
                self.reg_alloc.allocation.insert(*reg, Location::PhysReg(phys_regs[i].to_string()));
            } else {
                // Spill to stack
                let offset = -((i - phys_regs.len() + 1) as i32 * 8);
                self.reg_alloc.allocation.insert(*reg, Location::Stack(offset));
            }
        }
        
        // Calculate frame size
        let spilled = func.registers.len().saturating_sub(phys_regs.len());
        self.reg_alloc.frame_size = (spilled * 8) as u32;
    }
    
    /// Generate instruction
    fn generate_instruction(&mut self, insn: &Instruction) {
        match insn {
            Instruction::Nop => {}
            
            Instruction::Mov { dest, src } => {
                let dest_loc = self.reg_to_location(*dest);
                let src_str = self.value_to_string(src);
                self.emit_line(&format!("mov {}, {}", dest_loc, src_str));
            }
            
            Instruction::BinOp { op, dest, src1, src2 } => {
                let dest_loc = self.reg_to_location(*dest);
                let src1_loc = self.reg_to_location(*src1);
                let src2_str = self.value_to_string(src2);
                
                // x86-64: destination juga menjadi source pertama untuk most ops
                self.emit_line(&format!("mov {}, {}", dest_loc, src1_loc));
                
                let op_str = match op {
                    BinOp::Add => "add",
                    BinOp::Sub => "sub",
                    BinOp::Mul => "imul",
                    BinOp::Div => "idiv",
                    BinOp::And => "and",
                    BinOp::Or => "or",
                    BinOp::Xor => "xor",
                    _ => unimplemented!("Binary op"),
                };
                
                self.emit_line(&format!("{} {}, {}", op_str, dest_loc, src2_str));
            }
            
            Instruction::Jmp { target } => {
                self.emit_line(&format!("jmp {}", target));
            }
            
            Instruction::Jcc { cond, target } => {
                let cc = match cond {
                    Condition::Eq => "e",
                    Condition::Ne => "ne",
                    Condition::Lt => "l",
                    Condition::Gt => "g",
                    Condition::Le => "le",
                    Condition::Ge => "ge",
                };
                self.emit_line(&format!("j{} {}", cc, target));
            }
            
            Instruction::Cmp { src1, src2 } => {
                let src1_loc = self.reg_to_location(*src1);
                let src2_str = self.value_to_string(src2);
                self.emit_line(&format!("cmp {}, {}", src1_loc, src2_str));
            }
            
            Instruction::Call { func } => {
                let func_str = self.value_to_string(func);
                self.emit_line(&format!("call {}", func_str));
            }
            
            Instruction::Ret => {
                // Epilogue
                self.emit_line("mov rsp, rbp");
                self.emit_line("pop rbp");
                self.emit_line("ret");
            }
            
            Instruction::Label { name } => {
                self.emit_label(name);
            }
            
            _ => unimplemented!("Instruction generation"),
        }
    }
    
    /// Convert register to location string
    fn reg_to_location(&self, reg: Register) -> String {
        match self.reg_alloc.allocation.get(&reg) {
            Some(Location::PhysReg(r)) => format!("%{}", r),
            Some(Location::Stack(offset)) => format!("{}(%rbp)", offset),
            None => panic!("Register {} not allocated", reg.0),
        }
    }
    
    /// Convert value to string
    fn value_to_string(&self, value: &Value) -> String {
        match value {
            Value::Reg(reg) => self.reg_to_location(*reg),
            Value::Imm(imm) => format!("${}", imm),
            Value::Label(label) => label.clone(),
        }
    }
    
    /// Emit line dengan indentasi
    fn emit_line(&mut self, content: &str) {
        let indent = "    ".repeat(self.indent);
        self.output.push_str(&format!("{}{}\n", indent, content));
    }
    
    /// Emit label (tanpa indentasi)
    fn emit_label(&mut self, name: &str) {
        self.output.push_str(&format!("{}:\n", name));
    }
    
    /// Get generated assembly
    pub fn output(&self) -> &str {
        &self.output
    }
}

/// Object file writer
pub struct ObjectWriter {
    format: ObjectFormat,
    data: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectFormat {
    Elf,
    MachO,
    Coff,
}

impl ObjectWriter {
    /// Create new object writer
    pub fn new(format: ObjectFormat) -> Self {
        Self {
            format,
            data: Vec::new(),
        }
    }
    
    /// Write ELF object file
    pub fn write_elf(&mut self, path: &Path) -> Result<(), CodegenError> {
        unimplemented!("ELF writing not implemented")
    }
    
    /// Write Mach-O object file
    pub fn write_macho(&mut self, path: &Path) -> Result<(), CodegenError> {
        unimplemented!("Mach-O writing not implemented")
    }
    
    /// Write COFF object file
    pub fn write_coff(&mut self, path: &Path) -> Result<(), CodegenError> {
        unimplemented!("COFF writing not implemented")
    }
}

/// Linker
pub struct Linker {
    target: String,
    linker_cmd: String,
}

impl Linker {
    /// Create new linker
    pub fn new(target: String) -> Self {
        let linker_cmd = if target.contains("windows") {
            "link.exe".to_string()
        } else {
            "ld".to_string()
        };
        
        Self { target, linker_cmd }
    }
    
    /// Link object files
    pub fn link(&self, objects: &[&Path], output: &Path, libs: &[String]) -> Result<(), CodegenError> {
        // Build linker command
        let mut cmd = std::process::Command::new(&self.linker_cmd);
        
        for obj in objects {
            cmd.arg(obj.as_os_str());
        }
        
        for lib in libs {
            cmd.arg(format!("-l{}", lib));
        }
        
        cmd.arg("-o").arg(output);
        
        // Execute
        let status = cmd.status()
            .map_err(|e| CodegenError::Io(e))?;
        
        if !status.success() {
            return Err(CodegenError::Linking("Linking failed".to_string()));
        }
        
        Ok(())
    }
}

/// Code generation errors
#[derive(Debug)]
pub enum CodegenError {
    Io(std::io::Error),
    Linking(String),
    UnsupportedTarget(String),
}

impl std::fmt::Display for CodegenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CodegenError::Io(e) => write!(f, "IO error: {}", e),
            CodegenError::Linking(s) => write!(f, "Linking error: {}", s),
            CodegenError::UnsupportedTarget(t) => write!(f, "Unsupported target: {}", t),
        }
    }
}

impl std::error::Error for CodegenError {}
