use crate::hir::*;
use crate::scope::{RibKind, ScopeTree};
use crate::types::*;
use faxc_par as ast;
use faxc_util::{DefId, DefIdGenerator, Handler, Span};

fn ast_type_to_hir(ty: &ast::Type) -> Type {
    match ty {
        ast::Type::Unit => Type::Unit,
        ast::Type::Never => Type::Never,
        ast::Type::Path(_) => Type::Int,
        ast::Type::Generic(_, _) => Type::Int,
        ast::Type::Reference(ty, _) => Type::Ref(Box::new(ast_type_to_hir(ty)), false),
        ast::Type::Pointer(_, _) => Type::Int,
        ast::Type::Slice(ty) => Type::Slice(Box::new(ast_type_to_hir(ty))),
        ast::Type::Array(ty, size) => Type::Array(Box::new(ast_type_to_hir(ty)), *size),
        ast::Type::Tuple(tys) => Type::Tuple(tys.iter().map(ast_type_to_hir).collect()),
        ast::Type::Fn(params, ret) => Type::Fn(
            params.iter().map(ast_type_to_hir).collect(),
            Box::new(ast_type_to_hir(ret)),
        ),
        ast::Type::TraitObject(_) => Type::String,
        ast::Type::ImplTrait(_) => Type::Infer(InferId(0)),
        ast::Type::Inferred => Type::Infer(InferId(0)),
    }
}

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

    /// Error count
    error_count: usize,
}

impl<'a> SemanticAnalyzer<'a> {
    /// Create new analyzer
    pub fn new(
        type_context: &'a mut TypeContext,
        def_id_gen: &'a DefIdGenerator,
        handler: &'a mut Handler,
    ) -> Self {
        Self {
            type_context,
            scope_tree: ScopeTree::new(),
            def_id_gen,
            current_ret_type: None,
            loop_stack: Vec::new(),
            handler,
            error_count: 0,
        }
    }

    /// Report a type error
    pub fn type_error(&mut self, message: impl Into<String>, span: Span) {
        self.error_count += 1;
        use faxc_util::diagnostic::DiagnosticBuilder;
        DiagnosticBuilder::error(message)
            .span(span)
            .emit(&self.handler);
    }

    /// Check if there were any errors
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.error_count
    }

    /// Check if two types are unifiable, emit error if not
    pub fn unify_types(&mut self, expected: &Type, found: &Type, span: Span) -> bool {
        if expected == found {
            return true;
        }

        // Allow some coercions
        if let Type::Infer(_) = expected {
            return true; // Inference variable can accept any type
        }
        if let Type::Infer(_) = found {
            return true;
        }

        self.type_error(
            format!("type mismatch: expected {:?}, found {:?}", expected, found),
            span,
        );
        false
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
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(f.name, def_id);
                    let infer_id = self.type_context.new_infer_var();
                    self.type_context
                        .set_def_type(def_id, Type::Infer(infer_id));
                },
                ast::Item::Struct(s) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(s.name, def_id);
                    self.type_context.set_def_type(def_id, Type::Adt(def_id));
                },
                ast::Item::Enum(e) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(e.name, def_id);
                    self.type_context.set_def_type(def_id, Type::Adt(def_id));
                },
                ast::Item::Trait(t) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(t.name, def_id);
                },
                ast::Item::Impl(_imp) => {
                    let def_id = self.def_id_gen.next();
                    self.type_context.set_def_type(def_id, Type::Adt(def_id));
                },
                ast::Item::Const(c) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(c.name, def_id);
                    let infer_id = self.type_context.new_infer_var();
                    self.type_context
                        .set_def_type(def_id, Type::Infer(infer_id));
                },
                ast::Item::Static(s) => {
                    let def_id = self.def_id_gen.next();
                    self.scope_tree.add_binding(s.name, def_id);
                    let infer_id = self.type_context.new_infer_var();
                    self.type_context
                        .set_def_type(def_id, Type::Infer(infer_id));
                },
                ast::Item::Use(u) => {
                    let def_id = self.def_id_gen.next();
                    if let Some(seg) = u.path.segments.first() {
                        self.scope_tree.add_binding(seg.ident, def_id);
                    }
                },
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

        // Analyze parameters
        let mut params = Vec::new();
        let mut param_pats = Vec::new();
        for param in &item.params {
            let hir_ty = ast_type_to_hir(&param.ty);
            let pat = Pattern::Binding {
                name: param.name,
                ty: hir_ty.clone(),
                mutability: param.mutable,
            };
            param_pats.push(pat.clone());

            let def_id = self.def_id_gen.next();
            self.scope_tree.add_binding(param.name, def_id);
            self.type_context.set_def_type(def_id, hir_ty.clone());

            params.push(Param { pat, ty: hir_ty });
        }

        // Analyze body
        let body_expr = self.analyze_block(item.body)?;

        self.scope_tree.exit_scope();

        // Determine return type
        let ret_type = item
            .ret_type
            .as_ref()
            .map(ast_type_to_hir)
            .unwrap_or(Type::Unit);

        // Extract body into proper structure
        let body = Body {
            params: param_pats,
            value: body_expr,
        };

        Some(FnItem {
            def_id,
            name: item.name,
            generics: GenericParams::default(),
            params,
            ret_type,
            body,
            async_kw: item.async_kw,
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

        Some(Expr::Block { stmts, expr, ty })
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
            },
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
                                ast::ElseClause::If(next_i) => {
                                    Box::new(ast::Expr::If(ast::IfExpr {
                                        cond: Box::new(next_i.cond),
                                        then_block: next_i.then_block,
                                        else_block: None, // Simplified for deep nesting
                                    }))
                                },
                            }),
                        })),
                    }),
                })?;
                Some(Stmt::Expr(if_expr))
            },
            ast::Stmt::Expr(e) => {
                let expr = self.analyze_expr(e)?;
                Some(Stmt::Expr(expr))
            },
            _ => None,
        }
    }

    /// Analyze expression
    fn analyze_expr(&mut self, expr: ast::Expr) -> Option<Expr> {
        match expr {
            ast::Expr::Literal(lit) => self.analyze_literal(lit),
            ast::Expr::Path(path) => self.analyze_path(path),
            ast::Expr::Binary(bin) => self.analyze_binary(bin),
            ast::Expr::Unary(unary) => self.analyze_unary(unary),
            ast::Expr::If(if_expr) => self.analyze_if(if_expr),
            ast::Expr::Call(call) => self.analyze_call(call),
            ast::Expr::Block(block) => self.analyze_block(block),
            ast::Expr::Tuple(items) => self.analyze_tuple(items),
            ast::Expr::Array(items) => self.analyze_array(items),
            ast::Expr::Index(index_expr) => self.analyze_index(index_expr),
            ast::Expr::Field(field_expr) => self.analyze_field(field_expr),
            ast::Expr::Match(match_expr) => self.analyze_match(match_expr),
            ast::Expr::Return(ret) => self.analyze_return(ret),
            ast::Expr::Break(value, label) => self.analyze_break(value, label),
            ast::Expr::Continue(label) => self.analyze_continue(label),
            ast::Expr::MethodCall(method_call) => self.analyze_method_call(method_call),
            ast::Expr::Closure(closure) => self.analyze_closure(closure),
            ast::Expr::Assign(assign) => self.analyze_assign(assign),
            ast::Expr::CompoundAssign(compound) => self.analyze_compound_assign(compound),
            ast::Expr::Range(range) => self.analyze_range(range),
            ast::Expr::Cast(cast_expr, target_ty) => self.analyze_cast(cast_expr, target_ty),
            ast::Expr::Async(async_expr) => self.analyze_async(async_expr),
            ast::Expr::Await(await_expr) => self.analyze_await(await_expr),
            _ => None,
        }
    }

    /// Analyze method call
    fn analyze_method_call(&mut self, expr: ast::MethodCallExpr) -> Option<Expr> {
        let receiver = self.analyze_expr(*expr.receiver)?;

        let mut args = Vec::new();
        for arg in expr.call_args {
            if let Some(a) = self.analyze_expr(arg) {
                args.push(a);
            }
        }

        Some(Expr::Call {
            func: Box::new(Expr::Field {
                object: Box::new(receiver),
                field: DefId(0),
                ty: Type::Fn(vec![], Box::new(Type::Unit)),
            }),
            args,
            ty: Type::Unit,
        })
    }

    /// Analyze closure (lambda)
    fn analyze_closure(&mut self, expr: ast::ClosureExpr) -> Option<Expr> {
        self.scope_tree.enter_scope(RibKind::Block);

        let mut params = Vec::new();
        let mut param_tys = Vec::new();
        for param in &expr.params {
            let def_id = self.def_id_gen.next();
            self.scope_tree.add_binding(param.name, def_id);

            let param_hir_ty = ast_type_to_hir(&param.ty);
            self.type_context.set_def_type(def_id, param_hir_ty.clone());
            param_tys.push(param_hir_ty.clone());

            params.push(Pattern::Binding {
                name: param.name,
                ty: param_hir_ty,
                mutability: false,
            });
        }

        let body = self.analyze_expr(*expr.body)?;
        let body_ty = body.ty();

        self.scope_tree.exit_scope();

        let ty = Type::Fn(param_tys, Box::new(body_ty));

        Some(Expr::Literal {
            lit: Literal::Unit,
            ty,
        })
    }

    /// Analyze assignment
    fn analyze_assign(&mut self, expr: ast::AssignExpr) -> Option<Expr> {
        let place = self.analyze_expr(*expr.place)?;
        let value = self.analyze_expr(*expr.value)?;

        Some(Expr::Assign {
            place: Box::new(place),
            value: Box::new(value),
        })
    }

    /// Analyze compound assignment
    fn analyze_compound_assign(&mut self, expr: ast::CompoundAssignExpr) -> Option<Expr> {
        let place = self.analyze_expr(*expr.place)?;
        let place_ty = place.ty();
        let rhs = self.analyze_expr(*expr.value)?;

        let op = match expr.op {
            ast::BinOp::Add => BinOp::Add,
            ast::BinOp::Sub => BinOp::Sub,
            ast::BinOp::Mul => BinOp::Mul,
            ast::BinOp::Div => BinOp::Div,
            ast::BinOp::Mod => BinOp::Mod,
            _ => BinOp::Add,
        };

        Some(Expr::Binary {
            op,
            left: Box::new(place),
            right: Box::new(rhs),
            ty: place_ty,
        })
    }

    /// Analyze range expression
    fn analyze_range(&mut self, expr: ast::RangeExpr) -> Option<Expr> {
        let _start = expr.start.and_then(|s| self.analyze_expr(*s));
        let _end = expr.end.and_then(|e| self.analyze_expr(*e));

        let ty = Type::Slice(Box::new(Type::Int));

        Some(Expr::Literal {
            lit: Literal::Unit,
            ty,
        })
    }

    /// Analyze cast expression
    fn analyze_cast(&mut self, expr: Box<ast::Expr>, target_ty: ast::Type) -> Option<Expr> {
        let inner = self.analyze_expr(*expr)?;
        let ty = ast_type_to_hir(&target_ty);

        Some(Expr::Cast {
            expr: Box::new(inner),
            ty,
        })
    }

    /// Analyze async expression
    fn analyze_async(&mut self, expr: ast::AsyncExpr) -> Option<Expr> {
        let body = self.analyze_block(expr.body)?;
        let body_ty = body.ty();

        Some(Expr::Async {
            body: Box::new(body),
            ty: Type::Future(Box::new(body_ty)),
        })
    }

    /// Analyze await expression
    fn analyze_await(&mut self, expr: Box<ast::Expr>) -> Option<Expr> {
        let future = self.analyze_expr(*expr)?;

        let ty = match future.ty() {
            Type::Future(inner_ty) => *inner_ty,
            _ => Type::Unit,
        };

        Some(Expr::Await {
            expr: Box::new(future),
            ty,
        })
    }

    /// Analyze unary expression
    fn analyze_unary(&mut self, expr: ast::UnaryExpr) -> Option<Expr> {
        let inner = self.analyze_expr(*expr.expr)?;

        let op = match expr.op {
            ast::UnOp::Neg => UnOp::Neg,
            ast::UnOp::Not => UnOp::Not,
            ast::UnOp::BitNot => UnOp::Not, // Map to Not for now
            ast::UnOp::Deref => UnOp::Deref,
            ast::UnOp::Ref(mutable) => UnOp::Ref(mutable),
        };

        let ty = match inner.ty() {
            Type::Ref(inner_ty, _) => *inner_ty,
            _ => Type::Unit,
        };

        Some(Expr::Unary {
            op,
            expr: Box::new(inner),
            ty,
        })
    }

    /// Analyze function call
    fn analyze_call(&mut self, call: ast::CallExpr) -> Option<Expr> {
        let func = self.analyze_expr(*call.func)?;

        let mut args = Vec::new();
        for arg in call.args {
            if let Some(a) = self.analyze_expr(arg) {
                args.push(a);
            }
        }

        let ty = match func.ty() {
            Type::Fn(_, ret_ty) => *ret_ty,
            Type::Infer(_) => Type::Unit,
            _ => Type::Unit,
        };

        Some(Expr::Call {
            func: Box::new(func),
            args,
            ty,
        })
    }

    /// Analyze tuple
    fn analyze_tuple(&mut self, items: Vec<ast::Expr>) -> Option<Expr> {
        let mut analyzed = Vec::new();
        for item in items {
            if let Some(a) = self.analyze_expr(item) {
                analyzed.push(a);
            }
        }

        let ty = Type::Tuple(analyzed.iter().map(|e| e.ty()).collect());

        Some(Expr::Literal {
            lit: Literal::Unit,
            ty,
        })
    }

    /// Analyze array
    fn analyze_array(&mut self, items: Vec<ast::Expr>) -> Option<Expr> {
        let mut analyzed = Vec::new();
        for item in items {
            if let Some(a) = self.analyze_expr(item) {
                analyzed.push(a);
            }
        }

        let elem_ty = analyzed.first().map(|e| e.ty()).unwrap_or(Type::Unit);
        let ty = Type::Array(Box::new(elem_ty), analyzed.len());

        Some(Expr::Literal {
            lit: Literal::Unit,
            ty,
        })
    }

    /// Analyze index expression
    fn analyze_index(&mut self, index_expr: ast::IndexExpr) -> Option<Expr> {
        let object = self.analyze_expr(*index_expr.object)?;
        let index = self.analyze_expr(*index_expr.index)?;

        let ty = match object.ty() {
            Type::Array(elem_ty, _) => *elem_ty,
            Type::Slice(elem_ty) => *elem_ty,
            Type::Tuple(tys) => {
                if let Expr::Literal {
                    lit: Literal::Int(n),
                    ..
                } = index
                {
                    tys.get(n as usize).cloned().unwrap_or(Type::Unit)
                } else {
                    Type::Unit
                }
            },
            _ => Type::Unit,
        };

        Some(Expr::Literal {
            lit: Literal::Unit,
            ty,
        })
    }

    /// Analyze field access
    fn analyze_field(&mut self, field_expr: ast::FieldExpr) -> Option<Expr> {
        let object = self.analyze_expr(*field_expr.object)?;

        // Resolve field name to def_id (placeholder)
        let field = DefId(0);

        let ty = Type::Unit;

        Some(Expr::Field {
            object: Box::new(object),
            field,
            ty,
        })
    }

    /// Analyze match expression
    fn analyze_match(&mut self, match_expr: ast::MatchExpr) -> Option<Expr> {
        let scrutinee = self.analyze_expr(*match_expr.scrutinee)?;

        let mut arms = Vec::new();
        for arm in match_expr.arms {
            let pat = self.analyze_pattern(arm.pattern)?;
            let guard = arm.guard.and_then(|g| self.analyze_expr(g));
            let body = self.analyze_expr(arm.body)?;

            arms.push(Arm { pat, guard, body });
        }

        let ty = arms.first().map(|a| a.body.ty()).unwrap_or(Type::Unit);

        Some(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            ty,
        })
    }

    /// Analyze pattern
    fn analyze_pattern(&mut self, pat: ast::Pattern) -> Option<Pattern> {
        match pat {
            ast::Pattern::Wildcard => Some(Pattern::Wildcard),
            ast::Pattern::Ident(name, mutability) => {
                let ty = Type::Infer(InferId(0));
                Some(Pattern::Binding {
                    name,
                    ty,
                    mutability: matches!(mutability, ast::Mutability::Mutable),
                })
            },
            ast::Pattern::Literal(lit) => {
                let (_lit_kind, ty) = match lit {
                    ast::Literal::Int(n) => (Literal::Int(n), Type::Int),
                    ast::Literal::Float(f) => (Literal::Float(f), Type::Float),
                    ast::Literal::String(s) => (Literal::String(s), Type::String),
                    ast::Literal::Bool(b) => (Literal::Bool(b), Type::Bool),
                    ast::Literal::Char(c) => (Literal::Char(c), Type::Char),
                    ast::Literal::Unit => (Literal::Unit, Type::Unit),
                };
                Some(Pattern::Binding {
                    name: faxc_util::Symbol::intern("_"),
                    ty,
                    mutability: false,
                })
            },
            ast::Pattern::Path(path) => {
                let name = path.segments.first()?.ident;
                let def_id = self.scope_tree.resolve(name).unwrap_or(DefId(0));
                Some(Pattern::Path { def_id })
            },
            ast::Pattern::Tuple(pats) => {
                let mut analyzed = Vec::new();
                for p in pats {
                    if let Some(ap) = self.analyze_pattern(p) {
                        analyzed.push(ap);
                    }
                }
                Some(Pattern::Tuple { pats: analyzed })
            },
            _ => None,
        }
    }

    /// Analyze return expression
    fn analyze_return(&mut self, value: Option<Box<ast::Expr>>) -> Option<Expr> {
        let val = value.and_then(|v| self.analyze_expr(*v));
        Some(Expr::Return(val.map(Box::new)))
    }

    /// Analyze break expression
    fn analyze_break(
        &mut self,
        value: Option<Box<ast::Expr>>,
        label: Option<faxc_util::Symbol>,
    ) -> Option<Expr> {
        let val = value.and_then(|v| self.analyze_expr(*v));
        Some(Expr::Break(val.map(Box::new), label.map(|_| LabelId(0))))
    }

    /// Analyze continue expression
    fn analyze_continue(&mut self, label: Option<faxc_util::Symbol>) -> Option<Expr> {
        Some(Expr::Continue(label.map(|_| LabelId(0))))
    }

    /// Analyze if expression
    fn analyze_if(&mut self, expr: ast::IfExpr) -> Option<Expr> {
        let cond = self.analyze_expr(*expr.cond)?;

        // Condition must be bool
        if cond.ty() != Type::Bool {
            use faxc_util::diagnostic::DiagnosticBuilder;
            DiagnosticBuilder::error("If condition must be a boolean")
                .span(faxc_util::Span::DUMMY)
                .emit(&self.handler);
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
                use faxc_util::diagnostic::DiagnosticBuilder;
                DiagnosticBuilder::error("If and Else branches must have the same type")
                    .span(faxc_util::Span::DUMMY)
                    .emit(&self.handler);
            }
        } else {
            // If no else, type must be unit
            if then_expr.ty() != Type::Unit {
                use faxc_util::diagnostic::DiagnosticBuilder;
                DiagnosticBuilder::error("If branch without else must return unit")
                    .span(faxc_util::Span::DUMMY)
                    .emit(&self.handler);
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
        let ty = self
            .type_context
            .type_of_def(def_id)
            .cloned()
            .unwrap_or(Type::Int);

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
                use faxc_util::diagnostic::DiagnosticBuilder;
                DiagnosticBuilder::error("Bitwise XOR operator is not yet supported")
                    .span(span)
                    .emit(&self.handler);
                Some(BinOp::And) // Fallback to prevent compilation failure
            },
            ast::BinOp::Shl => {
                // Shift left not yet supported in HIR
                use faxc_util::diagnostic::DiagnosticBuilder;
                DiagnosticBuilder::error("Shift left operator is not yet supported")
                    .span(span)
                    .emit(&self.handler);
                Some(BinOp::Add) // Fallback to prevent compilation failure
            },
            ast::BinOp::Shr => {
                // Shift right not yet supported in HIR
                use faxc_util::diagnostic::DiagnosticBuilder;
                DiagnosticBuilder::error("Shift right operator is not yet supported")
                    .span(span)
                    .emit(&self.handler);
                Some(BinOp::Add) // Fallback to prevent compilation failure
            },
        }
    }
}
