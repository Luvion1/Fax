use crate::types::*;
use crate::hir::*;
use crate::scope::{ScopeTree, RibKind};
use faxc_par as ast;
use faxc_util::{Handler, DefIdGenerator, Span};

/// Main semantic analyzer
pub struct SemanticAnalyzer<'a> {
    /// Type context
    pub type_context: &'a mut TypeContext,

    /// Scope tree
    pub scope_tree: ScopeTree,

    /// Definition ID generator
    pub def_id_gen: &'a DefIdGenerator,

    /// Current function return type (for return checking)
    pub current_ret_type: Option<Type>,

    /// Loop stack (for break/continue checking)
    pub loop_stack: Vec<(Option<LabelId>, Type)>,

    /// Error handler
    pub handler: &'a mut Handler,
}

impl<'a> SemanticAnalyzer<'a> {
    /// Create new analyzer
    pub fn new(type_context: &'a mut TypeContext, def_id_gen: &'a DefIdGenerator, handler: &'a mut Handler) -> Self {
        Self {
            type_context,
            scope_tree: ScopeTree::new(),
            def_id_gen,
            current_ret_type: None,
            loop_stack: Vec::new(),
            handler,
        }
    }

    /// Analyze AST items and produce HIR
    pub fn analyze_items(&mut self, items: Vec<ast::Item>) -> Vec<Item> {
        println!("Analyzing {} items...", items.len());
        // First pass: collect all item names
        self.collect_items(&items);

        // Second pass: resolve and type check
        let hir_items: Vec<_> = items
            .into_iter()
            .filter_map(|item| {
                let res = self.analyze_item(item);
                if res.is_none() {
                    println!("analyze_item returned None");
                }
                res
            })
            .collect();

        println!("Generated {} HIR items.", hir_items.len());
        hir_items
    }

    /// Collect item names (first pass)
    fn collect_items(&mut self, items: &[ast::Item]) {
        for item in items {
            match item {
                ast::Item::Fn(f) => {
                    println!("Registering function: {}", f.name.as_str());
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(f.name, def_id);
                }
                ast::Item::Struct(s) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(s.name, def_id);
                }
                _ => {}
            }
        }
    }

    /// Analyze single item
    fn analyze_item(&mut self, item: ast::Item) -> Option<Item> {
        match item {
            ast::Item::Fn(fn_item) => {
                println!("Analyzing function item: {}", fn_item.name.as_str());
                self.analyze_fn_item(fn_item).map(Item::Function)
            },
            // Implement others as needed
            _ => {
                println!("Non-function item encountered");
                None
            },
        }
    }

    /// Analyze function item
    fn analyze_fn_item(&mut self, item: ast::FnItem) -> Option<FnItem> {
        let def_id = self.scope_tree.resolve(item.name)?;

        // Enter function scope
        self.scope_tree.enter_scope(RibKind::Function);

        // Analyze parameters (Placeholder for now)
        let params = vec![];

        // Analyze body
        let body_expr = self.analyze_block(item.body)?;

        self.scope_tree.exit_scope();

        // Extract body into proper structure
        let body = Body {
            params: vec![], // TODO: Handle patterns
            value: body_expr
        };

        Some(FnItem {
            def_id,
            name: item.name,
            generics: GenericParams::default(),
            params,
            ret_type: Type::Unit, // Default
            body,
            async_kw: false,
        })
    }

    /// Analyze block expression
    fn analyze_block(&mut self, block: ast::Block) -> Option<Expr> {
        self.scope_tree.enter_scope(RibKind::Block);

        let mut stmts = Vec::new();
        for stmt in block.stmts {
            if let Some(s) = self.analyze_stmt(stmt) {
                stmts.push(s);
            }
        }

        let mut ty = Type::Unit;
        let mut expr = None;

        if let Some(trailing) = block.trailing {
            if let Some(e) = self.analyze_expr(*trailing) {
                ty = e.ty();
                expr = Some(Box::new(e));
            }
        }

        self.scope_tree.exit_scope();

        Some(Expr::Block {
            stmts,
            expr,
            ty,
        })
    }

    /// Analyze statement
    fn analyze_stmt(&mut self, stmt: ast::Stmt) -> Option<Stmt> {
        match stmt {
            ast::Stmt::Let(l) => {
                let init = if let Some(expr) = l.init {
                    self.analyze_expr(expr)
                } else {
                    None
                };

                // Placeholder pattern handling
                let (name, mutability) = match l.pattern {
                    ast::Pattern::Ident(s, m) => (s, matches!(m, ast::Mutability::Mutable)),
                    _ => (faxc_util::Symbol::intern("unknown"), false),
                };

                // Register variable in scope using generator
                let def_id = self.def_id_gen.next();
                self.scope_tree.add_binding(name, def_id);
                self.type_context.set_def_type(def_id, Type::Int);

                let pat = Pattern::Binding {
                    name,
                    ty: Type::Int, // Inference placeholder
                    mutability,
                };

                Some(Stmt::Let {
                    pat,
                    ty: Type::Int,
                    init,
                })
            }
            ast::Stmt::If(if_stmt) => {
                // Convert Stmt::If to Expr::If wrapped in Stmt::Expr
                let if_expr = self.analyze_if(ast::IfExpr {
                    cond: Box::new(if_stmt.cond),
                    then_block: if_stmt.then_block,
                    else_block: if_stmt.else_clause.map(|c| match *c {
                        ast::ElseClause::Block(b) => Box::new(ast::Expr::Block(b)),
                        ast::ElseClause::If(i) => Box::new(ast::Expr::If(ast::IfExpr {
                            cond: Box::new(i.cond),
                            then_block: i.then_block,
                            else_block: i.else_clause.map(|ec| match *ec {
                                ast::ElseClause::Block(b) => Box::new(ast::Expr::Block(b)),
                                ast::ElseClause::If(next_i) => Box::new(ast::Expr::If(ast::IfExpr {
                                    cond: Box::new(next_i.cond),
                                    then_block: next_i.then_block,
                                    else_block: None, // Simplified for deep nesting
                                })),
                            }),
                        })),
                    }),
                })?;
                Some(Stmt::Expr(if_expr))
            }
            ast::Stmt::Expr(e) => {
                let expr = self.analyze_expr(e)?;
                Some(Stmt::Expr(expr))
            }
            _ => None,
        }
    }

    /// Analyze expression
    fn analyze_expr(&mut self, expr: ast::Expr) -> Option<Expr> {
        match expr {
            ast::Expr::Literal(lit) => self.analyze_literal(lit),
            ast::Expr::Path(path) => self.analyze_path(path),
            ast::Expr::Binary(bin) => self.analyze_binary(bin),
            ast::Expr::If(if_expr) => self.analyze_if(if_expr),
            _ => None,
        }
    }

    /// Analyze if expression
    fn analyze_if(&mut self, expr: ast::IfExpr) -> Option<Expr> {
        let cond = self.analyze_expr(*expr.cond)?;

        // Condition must be bool
        if cond.ty() != Type::Bool {
            self.handler.error("If condition must be a boolean".to_string(), faxc_util::Span::DUMMY);
        }

        let then_expr = Box::new(self.analyze_block(expr.then_block)?);

        let mut else_expr = None;
        let mut ty = Type::Unit;

        if let Some(e) = expr.else_block {
            let e_analyzed = self.analyze_expr(*e)?;
            ty = e_analyzed.ty();
            else_expr = Some(Box::new(e_analyzed));

            // Check type compatibility
            if then_expr.ty() != ty {
                self.handler.error("If and Else branches must have the same type".to_string(), faxc_util::Span::DUMMY);
            }
        } else {
            // If no else, type must be unit
            if then_expr.ty() != Type::Unit {
                self.handler.error("If branch without else must return unit".to_string(), faxc_util::Span::DUMMY);
            }
        }

        Some(Expr::If {
            cond: Box::new(cond),
            then_expr,
            else_expr,
            ty,
        })
    }

    /// Analyze literal
    fn analyze_literal(&mut self, lit: ast::Literal) -> Option<Expr> {
        let (lit_kind, ty) = match lit {
            ast::Literal::Int(n) => (Literal::Int(n), Type::Int),
            ast::Literal::Float(f) => (Literal::Float(f), Type::Float),
            ast::Literal::String(s) => (Literal::String(s), Type::String),
            ast::Literal::Bool(b) => (Literal::Bool(b), Type::Bool),
            ast::Literal::Char(c) => (Literal::Char(c), Type::Char),
            ast::Literal::Unit => (Literal::Unit, Type::Unit),
        };

        Some(Expr::Literal { lit: lit_kind, ty })
    }

    /// Analyze path expression
    fn analyze_path(&mut self, path: ast::Path) -> Option<Expr> {
        // Resolve path to definition
        let name = path.segments.first()?;
        let def_id = self.scope_tree.resolve(name.ident)?;

        // Get type of definition (Mocked for MVP if not in context)
        let ty = self.type_context.type_of_def(def_id).cloned().unwrap_or(Type::Int);

        Some(Expr::Var { def_id, ty })
    }

    /// Analyze binary expression
    fn analyze_binary(&mut self, expr: ast::BinaryExpr) -> Option<Expr> {
        let left = self.analyze_expr(*expr.left)?;
        let right = self.analyze_expr(*expr.right)?;

        let op = self.convert_binop(expr.op, expr.span)?;

        // Determine result type
        let ty = match op {
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => Type::Bool,
            _ => left.ty(),
        };

        Some(Expr::Binary {
            op,
            left: Box::new(left),
            right: Box::new(right),
            ty,
        })
    }

    fn convert_binop(&self, op: ast::BinOp, span: Span) -> Option<BinOp> {
        match op {
            ast::BinOp::Add => Some(BinOp::Add),
            ast::BinOp::Sub => Some(BinOp::Sub),
            ast::BinOp::Mul => Some(BinOp::Mul),
            ast::BinOp::Div => Some(BinOp::Div),
            ast::BinOp::Mod => Some(BinOp::Mod),
            ast::BinOp::Eq => Some(BinOp::Eq),
            ast::BinOp::Ne => Some(BinOp::Ne),
            ast::BinOp::Lt => Some(BinOp::Lt),
            ast::BinOp::Gt => Some(BinOp::Gt),
            ast::BinOp::Le => Some(BinOp::Le),
            ast::BinOp::Ge => Some(BinOp::Ge),
            ast::BinOp::And => Some(BinOp::And),
            ast::BinOp::Or => Some(BinOp::Or),
            ast::BinOp::BitAnd => Some(BinOp::And), // Map bitwise AND to logical AND for MVP
            ast::BinOp::BitOr => Some(BinOp::Or),   // Map bitwise OR to logical OR for MVP
            ast::BinOp::BitXor => {
                // Bitwise XOR not yet supported in HIR
                self.handler.error(
                    "Bitwise XOR operator is not yet supported".to_string(),
                    span
                );
                Some(BinOp::And) // Fallback to prevent compilation failure
            },
            ast::BinOp::Shl => {
                // Shift left not yet supported in HIR
                self.handler.error(
                    "Shift left operator is not yet supported".to_string(),
                    span
                );
                Some(BinOp::Add) // Fallback to prevent compilation failure
            },
            ast::BinOp::Shr => {
                // Shift right not yet supported in HIR
                self.handler.error(
                    "Shift right operator is not yet supported".to_string(),
                    span
                );
                Some(BinOp::Add) // Fallback to prevent compilation failure
            },
        }
    }
}
