//! faxc-par - Parser (Syntactic Analyzer)
//!
//! ============================================================================
//! PARSING THEORY
//! ============================================================================
//!
//! Parsing is the process of analyzing a string of tokens to determine
//! its grammatical structure according to a formal grammar. The output
//! is an Abstract Syntax Tree (AST).
//!
//! FORMAL DEFINITION:
//! ------------------
//! Given:
//! - G = (N, T, P, S) where
//!   N = non-terminal symbols
//!   T = terminal symbols (tokens)
//!   P = production rules
//!   S = start symbol
//!
//! Parsing is finding a derivation S ⇒* w where w is the input token sequence.
//!
//! GRAMMAR TYPES (Chomsky Hierarchy):
//! ----------------------------------
//!
//! Type 0: Unrestricted (Turing-complete)
//! Type 1: Context-sensitive
//! Type 2: Context-free (most programming languages)
//! Type 3: Regular (tokens/lexer)
//!
//! Most programming languages use context-free grammars (CFG) with some
//! context-sensitive elements (e.g., type checking) handled later.
//!
//! GRAMMAR NOTATION:
//! -----------------
//! We use Extended Backus-Naur Form (EBNF):
//!
//! ```ebnf
//! function = "fn" identifier "(" parameters ")" [ "->" type ] block ;
//!
//! parameters = [ parameter { "," parameter } ] ;
//!
//! parameter = identifier ":" type ;
//!
//! block = "{" { statement } "}" ;
//! ```
//!
//! Symbols:
//! - "literal" = exact token match
//! - [ optional ] = zero or one
//! - { repetition } = zero or more
//! - ( grouping ) = precedence
//! - | = alternation (choice)
//!
//! ============================================================================
//! PARSING ALGORITHMS
//! ============================================================================
//!
//! ALGORITHM 1: RECURSIVE DESCENT
//! ------------------------------
//!
//! A top-down parser where each non-terminal has a corresponding function.
//!
//! Structure:
//! ```
//! parse_function() {
//!     expect(Fn);
//!     name = parse_identifier();
//!     expect(LParen);
//!     params = parse_parameters();
//!     expect(RParen);
//!     ret = optional(parse_return_type);
//!     body = parse_block();
//!     return Function { name, params, ret, body };
//! }
//! ```
//!
//! REQUIREMENTS:
//! - Grammar must not be left-recursive
//! - Predictive (no backtracking) requires LL(1) grammar
//!
//! LEFT RECURSION ELIMINATION:
//! ---------------------------
//!
//! Problem:
//! ```
//! expr := expr + term | term
//! ```
//!
//! This causes infinite recursion in recursive descent!
//!
//! Solution - Transform to right recursion:
//! ```
//! expr := term { + term }
//! ```
//!
//! Or use iteration:
//! ```
//! parse_expr() {
//!     left = parse_term();
//!     while current == Plus {
//!         consume(Plus);
//!         right = parse_term();
//!         left = Binary(left, Plus, right);
//!     }
//!     return left;
//! }
//! ```
//!
//! ALGORITHM 2: PRATT PARSING (TOP-DOWN OPERATOR PRECEDENCE)
//! ---------------------------------------------------------
//!
//! Efficient expression parsing handling precedence and associativity.
//!
//! KEY IDEA:
//! Each token has two binding powers:
//! - Left binding power (lbp): How strongly it binds to the left
//! - Right binding power (rbp): How strongly it binds to the right
//!
//! Higher binding power = tighter grouping
//!
//! PRECEDENCE TABLE (higher number = tighter binding):
//! ```
//! Token       lbp     rbp     Associativity
//! -----------------------------------------
//! =           1       2       Right
//! ||          3       4       Left
//! &&          5       6       Left
//! ==, !=      7       8       Left
//! <, >, etc.  9       10      Left
//! +, -        11      12      Left
//! *, /, %     13      14      Left
//! !, - (unary)15      -       Prefix
//!
//! Literals    0       -       Atom
//! ```
//!
//! ALGORITHM:
//! ```
//! parse_expression(min_bp) {
//!     // Parse prefix (atom or prefix operator)
//!     lhs = parse_prefix();
//!     
//!     while lbp(current) >= min_bp {
//!         op = current;
//!         advance();
//!         rhs = parse_expression(rbp(op));
//!         lhs = Binary(lhs, op, rhs);
//!     }
//!     
//!     return lhs;
//! }
//! ```
//!
//! EXAMPLE:
//! ```
//! Input: a + b * c
//!
//! parse_expression(0):
//!   lhs = parse_prefix() → "a"
//!   
//!   lbp(+) = 11 >= 0, so:
//!     op = +
//!     advance()
//!     rhs = parse_expression(12)  // rbp(+) = 12
//!       parse_prefix() → "b"
//!       lbp(*) = 13 >= 12, so:
//!         op = *
//!         advance()
//!         rhs = parse_expression(14) → "c"
//!         lhs = Binary("b", *, "c")
//!       lbp(end) = 0 < 12, stop
//!       return Binary("b", *, "c")
//!     lhs = Binary("a", +, Binary("b", *, "c"))
//!   
//!   lbp(end) = 0 < 0, stop
//!   return Binary("a", +, Binary("b", *, "c"))
//! ```
//!
//! RESULT: Correct precedence: a + (b * c)
//!
//! ALGORITHM 3: LR PARSING (BOTTOM-UP)
//! -----------------------------------
//!
//! Not used in this implementation but important to understand.
//!
//! Uses a stack and state machine (DFA) to shift tokens and reduce
//! by production rules.
//!
//! Actions:
//! - Shift: Push token onto stack
//! - Reduce: Replace top N stack items with non-terminal
//!
//! Advantages:
//! - Handles left recursion naturally
//! - More powerful than LL (can parse more grammars)
//!
//! Disadvantages:
//! - Harder to write by hand
//! - Error messages less clear
//!
//! ============================================================================
//! ABSTRACT SYNTAX TREE (AST)
//! ============================================================================
//!
//! The AST represents the syntactic structure of code as a tree.
//! It abstracts away concrete syntax (parentheses, semicolons) and
//! focuses on semantic structure.
//!
//! DESIGN PRINCIPLES:
//! ------------------
//! 1. COMPLETENESS: Capture all semantic information
//! 2. ABSTRACTION: Remove syntactic sugar
//! 3. UNAMBIGUITY: One AST node per construct
//! 4. EXTENSIBILITY: Easy to add new node types
//!
//! AST vs CST (Concrete Syntax Tree):
//! ----------------------------------
//! CST includes all tokens (parentheses, braces, etc.).
//! AST is abstracted - only essential information.
//!
//! Example:
//! ```
//! Source: (a + b) * c
//!
//! CST:
//!   BinaryExpr
//!   ├── LParen "("
//!   ├── BinaryExpr
//!   │   ├── Ident "a"
//!   │   ├── Plus "+"
//!   │   └── Ident "b"
//!   ├── RParen ")"
//!   ├── Star "*"
//!   └── Ident "c"
//!
//! AST:
//!   BinaryExpr(*)
//!   ├── BinaryExpr(+)
//!   │   ├── Ident("a")
//!   │   └── Ident("b")
//!   └── Ident("c")
//! ```
//!
//! NODE TYPES:
//! -----------
//!
//! 1. ITEMS - Top-level declarations
//!    - Functions
//!    - Structs
//!    - Enums
//!    - Traits
//!    - Impl blocks
//!
//! 2. STATEMENTS - Executable code units
//!    - Let bindings
//!    - Expression statements
//!    - Control flow (if, while, for)
//!    - Return
//!
//! 3. EXPRESSIONS - Values and operations
//!    - Literals
//!    - Identifiers
//!    - Binary operations
//!    - Unary operations
//!    - Function calls
//!    - Field access
//!    - Block expressions
//!    - If/match expressions
//!    - Async block expressions
//!    - Await expressions
//!
//! 4. TYPES - Type expressions
//!    - Named types
//!    - Generic types
//!    - Reference types
//!    - Function types
//!    - Tuple types
//!
//! ============================================================================
//! ERROR RECOVERY
//! ============================================================================
//!
//! When parser encounters syntax error, it should:
//! 1. Report clear error message
//! 2. Recover to continue parsing
//! 3. Avoid cascading errors
//!
//! STRATEGY 1: PANIC MODE
//! ----------------------
//! Skip tokens until reaching synchronization point.
//!
//! Sync points:
//! - Statement separators (;)
//! - Block boundaries ({, })
//! - Top-level declarations
//!
//! ```
//! if x { y } else { z  // Missing }
//!
//! fn foo() { }         // Sync at 'fn'
//! ```
//!
//! STRATEGY 2: STATEMENT SKIPPING
//! ------------------------------
//! If error in statement, skip to next statement.
//!
//! STRATEGY 3: EXPECTED TOKEN INSERTION
//! ------------------------------------
//! Assume missing token exists and continue.
//!
//! ```
//! let x =           // Missing expression
//! let y = 10;
//!
//! Recovery: Insert dummy expression after '='
//! ```
//!
//! STRATEGY 4: DELIMITER MATCHING
//! ------------------------------
//! Match opening/closing delimiters intelligently.
//!
//! ```
//! { a + b           // Missing }
//!
//! Recovery: Insert } before next top-level item
//! ```
//!
//! ERROR MESSAGE QUALITY:
//! ----------------------
//! Good error messages should:
//! 1. Clearly state what was expected
//! 2. Show what was found instead
//! 3. Provide location (line, column)
//! 4. Suggest fix if obvious
//!
//! Example:
//! ```
//! error: expected `;`, found `let`
//!   --> main.fax:3:5
//!    |
//!  2 |     x = 5
//!    |          - help: consider adding `;` here
//!  3 |     let y = 10;
//!    |     ^^^ unexpected token
//! ```

// ============================================================================
// MACRO SYSTEM
// ============================================================================
//!
//! Fax supports three types of macros, similar to Rust:
//! 1. Declarative Macros (`macro_rules!`)
//! 2. Derive Macros (`#[derive(...)]`)
//! 3. Function-like Macros (`println!`, `vec![]`)
//!
//! ============================================================================
//! MACRO OVERVIEW
//! ============================================================================
//!
//! Macros provide compile-time code generation. They allow writing code
//! that writes other code, reducing boilerplate and enabling DSLs.
//!
//! KEY DIFFERENCE FROM FUNCTIONS:
//! - Functions: operate on values
//! - Macros: operate on code (tokens), expanded before compilation
//!
//! ============================================================================
//! DECLARATIVE MACROS (macro_rules!)
//! ============================================================================
//!
//! Declarative macros use pattern matching to generate code.
//!
//! SYNTAX:
//! -------
//! ```fax
//! macro_rules! macro_name {
//!     (pattern1) => { generated_code1 };
//!     (pattern2) => { generated_code2 };
//!     // ... more patterns
//! }
//! ```
//!
//! MACRO PATTERNS:
//! --------------
//! 1. LITERAL PATTERNS - Match exact tokens
//!    ```
//!    () => { ... }           // Match empty
//!    + => { ... }            // Match + operator
//!    fn => { ... }           // Match fn keyword
//!    ```
//!
//! 2. CAPTURE PATTERNS - Match and bind tokens
//!    ```
//!    $name:expr              // Match any expression, bind as "name"
//!    $name:ident             // Match any identifier
//!    $name:ty                // Match any type
//!    $name:pat              // Match any pattern
//!    $name:stmt             // Match any statement
//!    $name:block            // Match any block
//!    $name:meta             // Match any attribute/meta
//!    $name:item            // Match any item (fn, struct, etc.)
//!    $name:lifetime         // Match any lifetime
//!
//!    $name:tt               // Match any token tree
//!    $name:vis              // Match any visibility modifier
//!    ```
//!
//! 3. REPETITION PATTERNS - Match multiple tokens
//!    ```
//!    $($item:expr),*        // Zero or more, separated by comma
//!    $($item:expr)+         // One or more, separated by comma
//!    $($item:expr)?        // Optional (zero or one)
//!    $($item:expr),* $(,)? // Optional trailing comma
//!    ```
//!
//! EXAMPLE - vec![]:
//! -----------------
//! ```fax
//! macro_rules! vec {
//!     // vec![1, 2, 3]
//!     ($($item:expr),* $(,)?) => {
//!         {
//!             let mut temp_vec = ::std::Vec::new();
//!             $(
//!                 temp_vec.push($item);
//!             )*
//!             temp_vec
//!         }
//!     };
//! }
//! ```
//!
//! EXPANSION:
//! ----------
//! Given: `vec![1, 2, 3]`
//!
//! The macro expands to:
//! ```fax
//! {
//!     let mut temp_vec = ::std::Vec::new();
//!     temp_vec.push(1);
//!     temp_vec.push(2);
//!     temp_vec.push(3);
//!     temp_vec
//! }
//! ```
//!
//! EXAMPLE - map![]:
//! -----------------
//! ```fax
//! macro_rules! map {
//!     ($($key:expr => $value:expr),* $(,)?) => {{
//!         ::std::collections::HashMap::from([
//!             $(($key, $value)),*
//!         ])
//!     }};
//! }
//! ```
//!
//! Usage: `map!["a" => 1, "b" => 2]`
//!
//! EXAMPLE - html! DSL:
//! --------------------
//! ```fax
//! macro_rules! html {
//!     ($tag:ident $($attr:ident = $value:expr)*) => {{
//!         format!("<{}>", stringify!($tag))
//!     }};
//! }
//! ```
//!
//! ============================================================================
//! DERIVE MACROS
//! ============================================================================
//!
//! Derive macros generate trait implementations automatically.
//!
//! SYNTAX:
//! -------
//! ```fax
//! #[derive(Trait1, Trait2)]
//! struct MyStruct { ... }
//! ```
//!
//! BUILT-IN DERIVE TRAITS:
//! -----------------------
//!
//! 1. Clone - Generates `clone()` method
//!    ```fax
//!    #[derive(Clone)]
//!    struct Point { x: i32, y: i32 }
//!    ```
//!    Generates:
//!    ```fax
//!    impl Clone for Point {
//!        fn clone(&self) -> Point {
//!            Point { x: self.x, y: self.y }
//!        }
//!    }
//!    ```
//!
//! 2. Debug - Generates `fmt()` for debug formatting
//!    ```fax
//!    #[derive(Debug)]
//!    struct Point { x: i32, y: i32 }
//!    ```
//!    Generates:
//!    ```fax
//!    impl Debug for Point {
//!        fn fmt(&self, f: &mut Formatter) -> Result {
//!            write!(f, "Point {{ x: {}, y: {} }}", self.x, self.y)
//!        }
//!    }
//!    ```
//!
//! 3. PartialEq - Generates `==` and `!=` operators
//!    ```fax
//!    #[derive(PartialEq)]
//!    struct Point { x: i32, y: i32 }
//!    ```
//!
//! 4. Eq - Generates equality (requires PartialEq)
//!
//! 5. Default - Generates default constructor
//!    ```fax
//!    #[derive(Default)]
//!    struct Config { port: i32 = 8080 }
//!    ```
//!
//! 6. Copy - Generates bitwise copy (no heap allocation)
//!
//! ============================================================================
//! FUNCTION-LIKE MACROS
//! ============================================================================
//!
//! Macros that look like function calls but operate on tokens.
//!
//! BUILT-IN FUNCTION MACROS:
//! -------------------------
//!
//! 1. println! - Print with newline
//!    ```fax
//!    println!("Hello {}", name)
//!    println!("Number: {}", 42)
//!    println!("Multiple: {} and {}", a, b)
//!    ```
//!
//! 2. print! - Print without newline
//!    ```fax
//!    print!("Loading")
//!    print!("\rProgress: {}%", percent)
//!    ```
//!
//! 3. eprintln! - Print to stderr
//!    ```fax
//!    eprintln!("Error: {}", err)
//!    ```
//!
//! 4. format! - Create formatted string
//!    ```fax
//!    let s = format!("{} + {} = {}", a, b, a + b)
//!    ```
//!
//! 5. vec! - Create vector
//!    ```fax
//!    let v = vec![1, 2, 3]
//!    let empty: Vec<i32> = vec![]
//!    ```
//!
//! 6. assert! - Assert condition
//!    ```fax
//!    assert!(x > 0)
//!    assert!(result.is_ok(), "Error: {:?}", result)
//!    ```
//!
//! 7. assert_eq! - Assert equality
//!    ```fax
//!    assert_eq!(a, b)
//!    assert_eq!(result, expected, "custom message")
//!    ```
//!
//! 8. panic! - Panic with message
//!    ```fax
//!    panic!("Something went wrong")
//!    panic!("Expected {} but got {}", expected, got)
//!    ```
//!
//! ============================================================================
//! MACRO HYGIENE
//! ============================================================================
//!
//! Hygiene ensures macros don't accidentally capture or conflict with
//! variables from the calling context.
//!
//! THE PROBLEM:
//! ------------
//! Without hygiene, this macro would fail:
//! ```fax
//! macro_rules! double {
//!     ($x:expr) => { $x * 2 }
//! }
//!
//! fn main() {
//!     let x = 5;
//!     let result = double!(x);  // Should use the x above
//! }
//! ```
//!
//! HYGIENE SOLUTION:
//! -----------------
//! Each identifier created by a macro gets a unique "expansion context".
//! The macro's `x` is different from the caller's `x`.
//!
//! However, you can deliberately "break" hygiene using `$crate::variable`
//! or by passing identifiers that should refer to the caller's scope.
//!
//! ============================================================================
//! MACRO EXPANSION PIPELINE
//! ============================================================================
//!
//! ```
//! Source Code
//!      |
//!      v
//! +------------+
//! |   Lexer    |  Tokenize source
//! +------------+
//!      |
//!      v
//! +------------+
//! |   Parser   |  Parse tokens, recognize macro definitions
//! +------------+
//!      |
//!      v
//! +------------------+
//! | Macro Expansion |  Expand macros to generate code
//! +------------------+
//!      |  (recursive expansion until no macros left)
//!      v
//! +------------------+
//! |  Semantic (HIR) |  Continue normal compilation
//! +------------------+
//!      |
//!      v
//!     ...
//! ```
//!
//! ============================================================================
//! IMPLEMENTATION NOTES
//! ============================================================================
//!
//! 1. TOKEN TREE REPRESENTATION:
//!    - Macros work on token trees, not raw strings
//!    - TokenTree = Token | DelimitedGroup(TokenTree*)
//!    - Need to preserve parentheses, brackets, braces for grouping
//!
//! 2. MATCHING ALGORITHM:
//!    - Use recursive pattern matching
//!    - Handle repetitions with loop/recursion
//!    - Capture binding: store matched tokens per $name
//!
//! 3. EXPANSION:
//!    - Replace $name with captured tokens
//!    - Handle nested repetitions carefully
//!    - Output must be valid AST after expansion
//!
//! 4. ERROR HANDLING:
//!    - Report "no matching rule" clearly
//!    - Show which patterns were tried
//!    - Suggest corrections for common mistakes
//!
//! 5. RECURSIVE EXPANSION:
//!    - Macros can call other macros
//!    - Use worklist: keep expanding until no macros remain
//!    - Watch for infinite recursion (max expansion depth)

use faxc_lex::{Lexer, Token};
use faxc_util::{Diagnostic, Handler, Level, Span, Symbol};

// ============================================================================
// AST NODE DEFINITIONS
// ============================================================================

/// AST root - a source file contains a list of items
pub type Ast = Vec<Item>;

/// Top-level item in a source file
#[derive(Debug, Clone)]
pub enum Item {
    /// Function definition
    ///
    /// Example: `fn main() { println("hello"); }`
    Fn(FnItem),

    /// Structure definition
    ///
    /// Example: `struct Point { x: float, y: float }`
    Struct(StructItem),

    /// Enumeration definition
    ///
    /// Example: `enum Option<T> { Some(T), None }`
    Enum(EnumItem),

    /// Trait definition
    ///
    /// Example: `trait Printable { fn print(&self); }`
    Trait(TraitItem),

    /// Implementation block
    ///
    /// Example: `impl Trait for Type { ... }`
    Impl(ImplItem),

    /// Module import
    ///
    /// Example: `use std::io::println;`
    Use(UseItem),
}

/// Function item
#[derive(Debug, Clone)]
pub struct FnItem {
    /// Function name
    pub name: Symbol,

    /// Generic parameters (if any)
    ///
    /// Example: `<T, U: Display>`
    pub generics: Vec<GenericParam>,

    /// Function parameters
    pub params: Vec<Param>,

    /// Return type (None means unit type `()`)
    pub ret_type: Option<Type>,

    /// Function body
    pub body: Block,

    /// Visibility modifier
    pub visibility: Visibility,

    /// Source location
    pub span: Span,

    /// Async modifier (true if async function)
    pub async_kw: bool,
}

/// Generic parameter
#[derive(Debug, Clone)]
pub struct GenericParam {
    /// Parameter name (e.g., "T")
    pub name: Symbol,

    /// Trait bounds (e.g., "Display + Clone")
    pub bounds: Vec<Type>,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    /// Parameter name
    pub name: Symbol,

    /// Parameter type
    pub ty: Type,

    /// Mutability
    pub mutable: bool,
}

/// Structure item
#[derive(Debug, Clone)]
pub struct StructItem {
    /// Struct name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Fields
    pub fields: Vec<Field>,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,
}

/// Field definition (for structs and enums)
#[derive(Debug, Clone)]
pub struct Field {
    /// Field name
    pub name: Symbol,

    /// Field type
    pub ty: Type,

    /// Visibility (for struct fields)
    pub visibility: Visibility,
}

/// Enum item
#[derive(Debug, Clone)]
pub struct EnumItem {
    /// Enum name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Variants
    pub variants: Vec<Variant>,

    /// Visibility
    pub visibility: Visibility,

    /// Source location
    pub span: Span,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct Variant {
    /// Variant name
    pub name: Symbol,

    /// Variant data (unit, tuple, or struct)
    pub data: VariantData,
}

/// Variant data types
#[derive(Debug, Clone)]
pub enum VariantData {
    /// Unit variant (e.g., `None`)
    Unit,

    /// Tuple variant (e.g., `Some(T)`)
    Tuple(Vec<Type>),

    /// Struct variant (e.g., `Error { code: int, msg: string }`)
    Struct(Vec<Field>),
}

/// Trait item
#[derive(Debug, Clone)]
pub struct TraitItem {
    /// Trait name
    pub name: Symbol,

    /// Generic parameters
    pub generics: Vec<GenericParam>,

    /// Trait items (methods, types, constants)
    pub items: Vec<TraitMember>,

    /// Supertraits (e.g., `trait Foo: Bar + Baz`)
    pub supertraits: Vec<Type>,

    /// Visibility
    pub visibility: Visibility,
}

/// Trait member
#[derive(Debug, Clone)]
pub enum TraitMember {
    /// Method signature
    Method(FnSig),

    /// Associated type
    Type(Symbol, Vec<Type> /* bounds */),

    /// Associated constant
    Const(Symbol, Type, Option<Expr>),
}

/// Function signature (without body)
#[derive(Debug, Clone)]
pub struct FnSig {
    pub name: Symbol,
    pub generics: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
}

/// Implementation item
#[derive(Debug, Clone)]
pub struct ImplItem {
    /// Generic parameters for the impl
    pub generics: Vec<GenericParam>,

    /// Trait being implemented (None for inherent impl)
    pub trait_ref: Option<Type>,

    /// Type being implemented
    pub self_ty: Type,

    /// Items in the impl block
    pub items: Vec<ImplMember>,
}

/// Implementation member
#[derive(Debug, Clone)]
pub enum ImplMember {
    /// Method implementation
    Method(FnItem),

    /// Associated type definition
    Type(Symbol, Type),

    /// Associated constant
    Const(Symbol, Type, Expr),
}

/// Use/import item
#[derive(Debug, Clone)]
pub struct UseItem {
    /// Import path
    pub path: Path,

    /// Alias (if any)
    pub alias: Option<Symbol>,

    /// Glob import (*)
    pub is_glob: bool,
}

/// Visibility modifier
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// Public visibility (`pub`)
    Public,

    /// Private (default)
    Private,

    /// Public within crate (`pub(crate)`)
    Crate,

    /// Public within module (`pub(super)`)
    Super,

    /// Public within path (`pub(in path)`)
    Restricted(Path),
}

// ============================================================================
// STATEMENTS
// ============================================================================

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    /// Let binding
    ///
    /// Example: `let x: int = 5;`
    Let(LetStmt),

    /// Expression statement
    ///
    /// Example: `foo();` or `x = 5;`
    Expr(Expr),

    /// Return statement
    ///
    /// Example: `return x;` or `return;`
    Return(Option<Expr>),

    /// If statement (not expression form)
    ///
    /// Example: `if x { do_something(); }`
    If(IfStmt),

    /// While loop
    ///
    /// Example: `while condition { body; }`
    While(WhileStmt),

    /// For loop
    ///
    /// Example: `for x in iterable { body; }`
    For(ForStmt),

    /// Item statement (item inside function)
    ///
    /// Example: Nested function or struct definition
    Item(Item),
}

/// Let statement
#[derive(Debug, Clone)]
pub struct LetStmt {
    /// Pattern being bound
    ///
    /// Can be identifier, tuple, or destructuring pattern
    pub pattern: Pattern,

    /// Type annotation (optional)
    pub ty: Option<Type>,

    /// Initializer expression (optional)
    pub init: Option<Expr>,
}

/// If statement (statement form, not expression)
#[derive(Debug, Clone)]
pub struct IfStmt {
    /// Condition
    pub cond: Expr,

    /// Then block
    pub then_block: Block,

    /// Else clause (optional)
    pub else_clause: Option<Box<ElseClause>>,
}

/// Else clause
#[derive(Debug, Clone)]
pub enum ElseClause {
    /// Else block
    ///
    /// `else { block }`
    Block(Block),

    /// Else-if
    ///
    /// `else if cond { block }`
    If(IfStmt),
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileStmt {
    /// Condition
    pub cond: Expr,

    /// Loop body
    pub body: Block,

    /// Label (for break/continue)
    pub label: Option<Symbol>,
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForStmt {
    /// Pattern binding the iteration variable
    pub pattern: Pattern,

    /// Expression being iterated
    pub iter: Expr,

    /// Loop body
    pub body: Block,

    /// Label
    pub label: Option<Symbol>,
}

/// Block expression
#[derive(Debug, Clone)]
pub struct Block {
    /// Statements in the block
    pub stmts: Vec<Stmt>,

    /// Trailing expression (if block ends with expression)
    ///
    /// Example: `{ let x = 5; x + 1 }` has trailing expr `x + 1`
    pub trailing: Option<Box<Expr>>,

    /// Source location
    pub span: Span,
}

// ============================================================================
// EXPRESSIONS
// ============================================================================

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),

    /// Variable reference
    Path(Path),

    /// Binary operation
    Binary(BinaryExpr),

    /// Unary operation
    Unary(UnaryExpr),

    /// Function call
    Call(CallExpr),

    /// Method call
    MethodCall(MethodCallExpr),

    /// Field access
    Field(FieldExpr),

    /// Index expression
    Index(IndexExpr),

    /// Block expression
    Block(Block),

    /// If expression
    If(IfExpr),

    /// Match expression
    Match(MatchExpr),

    /// Closure expression
    Closure(ClosureExpr),

    /// Assignment expression
    Assign(AssignExpr),

    /// Compound assignment (+=, -=, etc.)
    CompoundAssign(CompoundAssignExpr),

    /// Return expression (within function)
    Return(Option<Box<Expr>>),

    /// Break expression (within loop)
    Break(Option<Box<Expr>>, Option<Symbol> /* label */),

    /// Continue expression (within loop)
    Continue(Option<Symbol> /* label */),

    /// Tuple expression
    Tuple(Vec<Expr>),

    /// Array expression
    Array(Vec<Expr>),

    /// Range expression
    Range(RangeExpr),

    /// Cast expression
    Cast(Box<Expr>, Type),

    /// Async block expression
    ///
    /// Example: `async { ... }`
    Async(AsyncExpr),

    /// Await expression
    ///
    /// Example: `await future`
    Await(Box<Expr>),
}

/// Literal expression
#[derive(Debug, Clone)]
pub enum Literal {
    /// Integer literal
    Int(i64),

    /// Float literal
    Float(f64),

    /// String literal
    String(Symbol),

    /// Boolean literal
    Bool(bool),

    /// Unit literal `()`
    Unit,
}

/// Path expression (variable or path)
#[derive(Debug, Clone)]
pub struct Path {
    /// Path segments
    ///
    /// Example: `std::io::println` has segments ["std", "io", "println"]
    pub segments: Vec<PathSegment>,
}

/// Path segment
#[derive(Debug, Clone)]
pub struct PathSegment {
    /// Segment name
    pub ident: Symbol,

    /// Generic arguments (if any)
    ///
    /// Example: `Vec<int>`
    pub args: Option<Vec<Type>>,
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub op: BinOp,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy)]
pub enum BinOp {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    // Logical
    And,
    Or,
    // Bitwise
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnOp,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy)]
pub enum UnOp {
    /// Negation (`-x`)
    Neg,

    /// Logical not (`!x`)
    Not,

    /// Bitwise not (`~x`)
    BitNot,

    /// Dereference (`*x`)
    Deref,

    /// Reference (`&x` or `&mut x`)
    Ref(bool /* mutable */),
}

/// Function call expression
#[derive(Debug, Clone)]
pub struct CallExpr {
    /// Function being called
    pub func: Box<Expr>,

    /// Arguments
    pub args: Vec<Expr>,

    /// Source location
    pub span: Span,
}

/// Method call expression
#[derive(Debug, Clone)]
pub struct MethodCallExpr {
    /// Receiver (object before the dot)
    pub receiver: Box<Expr>,

    /// Method name
    pub method: Symbol,

    /// Generic arguments (if turbofish: `method::<T>()`)
    pub args: Option<Vec<Type>>,

    /// Call arguments
    pub call_args: Vec<Expr>,
}

/// Field access expression
#[derive(Debug, Clone)]
pub struct FieldExpr {
    /// Object being accessed
    pub object: Box<Expr>,

    /// Field name
    pub field: Symbol,

    /// Source location
    pub span: Span,
}

/// Index expression
#[derive(Debug, Clone)]
pub struct IndexExpr {
    /// Array/vector being indexed
    pub object: Box<Expr>,

    /// Index expression
    pub index: Box<Expr>,
}

/// If expression (value-producing)
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub cond: Box<Expr>,
    pub then_block: Block,
    pub else_block: Option<Box<Expr>>,
}

/// Match expression
#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Box<Expr>,
    pub arms: Vec<MatchArm>,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub guard: Option<Expr>,
    pub body: Expr,
}

/// Closure expression
#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<Param>,
    pub ret_type: Option<Type>,
    pub body: Box<Expr>,
    pub move_kw: bool, // `move |...| ...`
}

/// Async expression
///
/// Represents an async block: `async { ... }` or `async move { ... }`
#[derive(Debug, Clone)]
pub struct AsyncExpr {
    pub body: Block,
    pub move_kw: bool, // `async move { ... }`
}

/// Assignment expression
#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub place: Box<Expr>,
    pub value: Box<Expr>,
}

/// Compound assignment expression
#[derive(Debug, Clone)]
pub struct CompoundAssignExpr {
    pub place: Box<Expr>,
    pub op: BinOp,
    pub value: Box<Expr>,
}

/// Range expression
#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool, // true for ..=, false for ..
}

// ============================================================================
// PATTERNS
// ============================================================================

/// Pattern (used in let, match, function params)
#[derive(Debug, Clone)]
pub enum Pattern {
    /// Wildcard pattern `_`
    Wildcard,

    /// Identifier pattern (binding)
    Ident(Symbol, Mutability),

    /// Literal pattern
    Literal(Literal),

    /// Path pattern (enum variant, constant)
    Path(Path),

    /// Struct pattern
    Struct(Path, Vec<FieldPattern>),

    /// Tuple struct pattern
    TupleStruct(Path, Vec<Pattern>),

    /// Tuple pattern
    Tuple(Vec<Pattern>),

    /// Array/slice pattern
    Slice(Vec<Pattern>),

    /// Reference pattern
    Ref(Box<Pattern>, Mutability),

    /// Mut pattern
    Mut(Box<Pattern>),

    /// Range pattern
    Range(Box<Pattern>, Box<Pattern>),

    /// Or pattern (|)
    Or(Vec<Pattern>),
}

/// Field in struct pattern
#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub field: Symbol,
    pub pattern: Pattern,
}

// ============================================================================
// TYPES
// ============================================================================

/// Type expression
#[derive(Debug, Clone)]
pub enum Type {
    /// Unit type `()`
    Unit,

    /// Never type `!`
    Never,

    /// Path type (named type)
    Path(Path),

    /// Generic type instantiation
    Generic(Box<Type>, Vec<Type>),

    /// Reference type `&T` or `&mut T`
    Reference(Box<Type>, Mutability),

    /// Pointer type `*const T` or `*mut T` (unsafe)
    Pointer(Box<Type>, Mutability),

    /// Slice type `[T]`
    Slice(Box<Type>),

    /// Array type `[T; N]`
    Array(Box<Type>, usize),

    /// Tuple type `(T1, T2, ...)`
    Tuple(Vec<Type>),

    /// Function type `fn(A, B) -> C`
    Fn(Vec<Type>, Box<Type>),

    /// Trait object type `dyn Trait`
    TraitObject(Vec<Type>),

    /// Impl trait type `impl Trait` (existential)
    ImplTrait(Vec<Type>),

    /// Inferred type `_`
    Inferred,
}

/// Mutability
#[derive(Debug, Clone, Copy)]
pub enum Mutability {
    Mutable,
    Immutable,
}

// ============================================================================
// PARSER STRUCTURE
// ============================================================================

/// Recursive descent parser
pub struct Parser<'a> {
    /// Token stream from lexer
    tokens: Vec<Token>,

    /// Current position in token stream
    position: usize,

    /// Previous token position (for error reporting)
    prev_position: usize,

    /// Error handler
    handler: &'a mut Handler,
}

impl<'a> Parser<'a> {
    /// Create a new parser from tokens
    pub fn new(tokens: Vec<Token>, handler: &'a mut Handler) -> Self {
        Self {
            tokens,
            position: 0,
            prev_position: 0,
            handler,
        }
    }

    /// Parse a complete source file
    pub fn parse(&mut self) -> Ast {
        let mut items = Vec::new();

        while !self.is_at_end() {
            match self.parse_item() {
                Some(item) => items.push(item),
                None => self.recover_to_sync_point(),
            }
        }

        items
    }

    /// Parse a single item
    fn parse_item(&mut self) -> Option<Item> {
        let visibility = self.parse_visibility();

        match self.current_token() {
            Token::Fn => self.parse_fn_item(visibility),
            Token::Struct => self.parse_struct_item(visibility),
            Token::Enum => self.parse_enum_item(visibility),
            Token::Impl => self.parse_impl_item(),
            Token::Trait => self.parse_trait_item(visibility),
            Token::Use => self.parse_use_item(),
            _ => {
                self.error("expected item".to_string());
                None
            }
        }
    }

    /// Parse function item
    fn parse_fn_item(&mut self, visibility: Visibility) -> Option<Item> {
        unimplemented!("Function parsing not implemented")
    }

    /// Parse struct item
    fn parse_struct_item(&mut self, visibility: Visibility) -> Option<Item> {
        unimplemented!("Struct parsing not implemented")
    }

    /// Parse enum item
    fn parse_enum_item(&mut self, visibility: Visibility) -> Option<Item> {
        unimplemented!("Enum parsing not implemented")
    }

    /// Parse impl item
    fn parse_impl_item(&mut self) -> Option<Item> {
        unimplemented!("Impl parsing not implemented")
    }

    /// Parse trait item
    fn parse_trait_item(&mut self, visibility: Visibility) -> Option<Item> {
        unimplemented!("Trait parsing not implemented")
    }

    /// Parse use item
    fn parse_use_item(&mut self) -> Option<Item> {
        unimplemented!("Use parsing not implemented")
    }

    /// Parse statement
    fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token() {
            Token::Let => self.parse_let_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            _ => {
                // Try expression statement
                let expr = self.parse_expr()?;
                self.expect(Token::Semicolon)?;
                Some(Stmt::Expr(expr))
            }
        }
    }

    /// Parse let statement
    fn parse_let_stmt(&mut self) -> Option<Stmt> {
        unimplemented!("Let statement parsing not implemented")
    }

    /// Parse if statement
    fn parse_if_stmt(&mut self) -> Option<Stmt> {
        unimplemented!("If statement parsing not implemented")
    }

    /// Parse while statement
    fn parse_while_stmt(&mut self) -> Option<Stmt> {
        unimplemented!("While statement parsing not implemented")
    }

    /// Parse for statement
    fn parse_for_stmt(&mut self) -> Option<Stmt> {
        unimplemented!("For statement parsing not implemented")
    }

    /// Parse return statement
    fn parse_return_stmt(&mut self) -> Option<Stmt> {
        unimplemented!("Return statement parsing not implemented")
    }

    /// Parse expression using Pratt parser
    fn parse_expr(&mut self) -> Option<Expr> {
        self.parse_expr_with_min_bp(0)
    }

    /// Parse expression with minimum binding power (Pratt parser)
    fn parse_expr_with_min_bp(&mut self, min_bp: u8) -> Option<Expr> {
        // Parse prefix (atom or prefix operator)
        let mut lhs = self.parse_prefix()?;

        // Parse infix operators with sufficient binding power
        loop {
            let (lbp, rbp) = match self.infix_binding_power() {
                Some(bp) if bp.0 >= min_bp => bp,
                _ => break,
            };

            let op = self.current_token();
            self.advance();

            let rhs = self.parse_expr_with_min_bp(rbp)?;
            lhs = Expr::Binary(BinaryExpr {
                left: Box::new(lhs),
                op: self.token_to_binop(op)?,
                right: Box::new(rhs),
                span: Span::DUMMY, // Calculate actual span
            });
        }

        Some(lhs)
    }

    /// Parse prefix expression (atom or prefix operator)
    fn parse_prefix(&mut self) -> Option<Expr> {
        match self.current_token() {
            // Prefix operators
            Token::Minus => self.parse_unary(UnOp::Neg),
            Token::Not => self.parse_unary(UnOp::Not),
            Token::Star => self.parse_unary(UnOp::Deref),
            Token::Ampersand => self.parse_reference(),

            // Atoms
            Token::Number(n) => {
                self.advance();
                Some(Expr::Literal(Literal::Int(n as i64)))
            }
            Token::String(s) => {
                self.advance();
                Some(Expr::Literal(Literal::String(s)))
            }
            Token::True => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(true)))
            }
            Token::False => {
                self.advance();
                Some(Expr::Literal(Literal::Bool(false)))
            }
            Token::Ident(_) => self.parse_path_or_call(),
            Token::LParen => self.parse_paren_or_tuple(),
            Token::LBrace => self.parse_block_expr(),
            Token::If => self.parse_if_expr(),
            Token::Match => self.parse_match_expr(),

            _ => {
                self.error("expected expression".to_string());
                None
            }
        }
    }

    /// Parse unary expression
    fn parse_unary(&mut self, op: UnOp) -> Option<Expr> {
        unimplemented!("Unary parsing not implemented")
    }

    /// Parse reference expression (& or &mut)
    fn parse_reference(&mut self) -> Option<Expr> {
        unimplemented!("Reference parsing not implemented")
    }

    /// Parse path or function call
    fn parse_path_or_call(&mut self) -> Option<Expr> {
        unimplemented!("Path/call parsing not implemented")
    }

    /// Parse parenthesized expression or tuple
    fn parse_paren_or_tuple(&mut self) -> Option<Expr> {
        unimplemented!("Paren/tuple parsing not implemented")
    }

    /// Parse block expression
    fn parse_block_expr(&mut self) -> Option<Expr> {
        unimplemented!("Block parsing not implemented")
    }

    /// Parse if expression
    fn parse_if_expr(&mut self) -> Option<Expr> {
        unimplemented!("If expression parsing not implemented")
    }

    /// Parse match expression
    fn parse_match_expr(&mut self) -> Option<Expr> {
        unimplemented!("Match expression parsing not implemented")
    }

    /// Parse visibility modifier
    fn parse_visibility(&mut self) -> Visibility {
        if self.match_token(Token::Pub) {
            // Check for restricted visibility: pub(crate), pub(super), pub(in path)
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    /// Parse type expression
    fn parse_type(&mut self) -> Option<Type> {
        unimplemented!("Type parsing not implemented")
    }

    /// Parse pattern
    fn parse_pattern(&mut self) -> Option<Pattern> {
        unimplemented!("Pattern parsing not implemented")
    }

    /// Get current token
    fn current_token(&self) -> Token {
        self.tokens
            .get(self.position)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    /// Check if at end of tokens
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    /// Advance to next token
    fn advance(&mut self) {
        if !self.is_at_end() {
            self.prev_position = self.position;
            self.position += 1;
        }
    }

    /// Expect specific token
    fn expect(&mut self, expected: Token) -> Option<()> {
        if self.current_token() == expected {
            self.advance();
            Some(())
        } else {
            self.error(format!("expected {:?}", expected));
            None
        }
    }

    /// Match and consume token
    fn match_token(&mut self, expected: Token) -> bool {
        if self.current_token() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Get infix operator binding power
    fn infix_binding_power(&self) -> Option<(u8, u8)> {
        // Returns (left_bp, right_bp)
        // Higher number = tighter binding
        match self.current_token() {
            Token::Eq => Some((1, 2)),   // Right associative
            Token::OrOr => Some((3, 4)), // Left associative
            Token::AndAnd => Some((5, 6)),
            Token::EqEq | Token::NotEq => Some((7, 8)),
            Token::Lt | Token::Gt | Token::LtEq | Token::GtEq => Some((9, 10)),
            Token::Plus | Token::Minus => Some((11, 12)),
            Token::Star | Token::Slash | Token::Percent => Some((13, 14)),
            _ => None,
        }
    }

    /// Convert token to binary operator
    fn token_to_binop(&self, token: Token) -> Option<BinOp> {
        match token {
            Token::Plus => Some(BinOp::Add),
            Token::Minus => Some(BinOp::Sub),
            Token::Star => Some(BinOp::Mul),
            Token::Slash => Some(BinOp::Div),
            Token::Percent => Some(BinOp::Mod),
            Token::EqEq => Some(BinOp::Eq),
            Token::NotEq => Some(BinOp::Ne),
            Token::Lt => Some(BinOp::Lt),
            Token::Gt => Some(BinOp::Gt),
            Token::LtEq => Some(BinOp::Le),
            Token::GtEq => Some(BinOp::Ge),
            Token::AndAnd => Some(BinOp::And),
            Token::OrOr => Some(BinOp::Or),
            _ => None,
        }
    }

    /// Report error
    fn error(&mut self, message: String) {
        // Create and emit diagnostic
        unimplemented!("Error reporting not implemented")
    }

    /// Recover to synchronization point
    fn recover_to_sync_point(&mut self) {
        // Skip tokens until we reach a sync point
        // Sync points: item keywords, closing braces, semicolons
        unimplemented!("Error recovery not implemented")
    }
}
