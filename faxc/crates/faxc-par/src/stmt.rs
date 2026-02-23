//! Statement parsing - let, if, while, for, return, etc.

use crate::ast::*;
use crate::Parser;
use faxc_lex::Token;

impl<'a> Parser<'a> {
    /// Parse a statement
    pub fn parse_stmt(&mut self) -> Option<Stmt> {
        match self.current_token() {
            Token::Let => self.parse_let_stmt(),
            Token::If => self.parse_if_stmt(),
            Token::While => self.parse_while_stmt(),
            Token::For => self.parse_for_stmt(),
            Token::Return => self.parse_return_stmt(),
            Token::Break => self.parse_break_stmt(),
            Token::Continue => self.parse_continue_stmt(),
            Token::LBrace => {
                let block = self.parse_block()?;
                Some(Stmt::Expr(Expr::Block(block)))
            },
            _ => {
                let expr = self.parse_expr()?;

                if self.match_token(Token::Eq) {
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon);
                    return Some(Stmt::Expr(Expr::Assign(AssignExpr {
                        place: Box::new(expr),
                        value: Box::new(value),
                    })));
                }

                if let Some(op) = self.parse_compound_assign_op() {
                    let value = self.parse_expr()?;
                    self.expect(Token::Semicolon);
                    return Some(Stmt::Expr(Expr::CompoundAssign(CompoundAssignExpr {
                        place: Box::new(expr),
                        op,
                        value: Box::new(value),
                    })));
                }

                if self.match_token(Token::Semicolon) {
                    Some(Stmt::Expr(expr))
                } else if self.is_at_end() || self.current_token() == Token::RBrace {
                    Some(Stmt::Expr(expr))
                } else {
                    self.expect(Token::Semicolon);
                    Some(Stmt::Expr(expr))
                }
            },
        }
    }

    /// Parse compound assignment operator
    pub fn parse_compound_assign_op(&mut self) -> Option<BinOp> {
        match self.current_token() {
            Token::PlusEq => {
                self.advance();
                Some(BinOp::Add)
            },
            Token::MinusEq => {
                self.advance();
                Some(BinOp::Sub)
            },
            Token::StarEq => {
                self.advance();
                Some(BinOp::Mul)
            },
            Token::SlashEq => {
                self.advance();
                Some(BinOp::Div)
            },
            Token::PercentEq => {
                self.advance();
                Some(BinOp::Mod)
            },
            Token::AmpersandEq => {
                self.advance();
                Some(BinOp::BitAnd)
            },
            Token::PipeEq => {
                self.advance();
                Some(BinOp::BitOr)
            },
            Token::CaretEq => {
                self.advance();
                Some(BinOp::BitXor)
            },
            Token::ShlEq => {
                self.advance();
                Some(BinOp::Shl)
            },
            Token::ShrEq => {
                self.advance();
                Some(BinOp::Shr)
            },
            _ => None,
        }
    }

    /// Parse let statement
    pub fn parse_let_stmt(&mut self) -> Option<Stmt> {
        let _span_start = self.current_span();

        self.expect(Token::Let)?;

        let mutable = self.match_token(Token::Mut);
        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(Token::Colon) {
            self.parse_type()
        } else {
            None
        };

        let init = if self.match_token(Token::Eq) {
            self.parse_expr()
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Let(LetStmt {
            pattern,
            ty,
            init,
            mutable,
        }))
    }

    /// Parse if statement
    pub fn parse_if_stmt(&mut self) -> Option<Stmt> {
        let _span_start = self.current_span();

        self.expect(Token::If)?;

        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_clause = if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                if let Some(Stmt::If(if_stmt)) = self.parse_if_stmt() {
                    Some(Box::new(ElseClause::If(if_stmt)))
                } else {
                    None
                }
            } else {
                let block = self.parse_block()?;
                Some(Box::new(ElseClause::Block(block)))
            }
        } else {
            None
        };

        Some(Stmt::If(IfStmt {
            cond,
            then_block,
            else_clause,
        }))
    }

    /// Parse while statement
    pub fn parse_while_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::While)?;

        let cond = self.parse_expr()?;
        let body = self.parse_block()?;

        Some(Stmt::While(WhileStmt {
            cond,
            body,
            label: None,
        }))
    }

    /// Parse for statement
    pub fn parse_for_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::For)?;

        let pattern = self.parse_pattern()?;

        let is_in = match self.current_token() {
            Token::Ident(sym) => sym.as_str() == "in",
            _ => false,
        };
        if !is_in {
            self.error("expected 'in' after pattern in for loop");
            return None;
        }
        self.advance();

        let iter = self.parse_expr()?;
        let body = self.parse_block()?;

        Some(Stmt::For(ForStmt {
            pattern,
            iter,
            body,
            label: None,
        }))
    }

    /// Parse return statement
    pub fn parse_return_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Return)?;

        let expr = if self.current_token() != Token::Semicolon
            && self.current_token() != Token::RBrace
            && !self.is_at_end()
        {
            self.parse_expr()
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Return(expr))
    }

    /// Parse break statement
    pub fn parse_break_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Break)?;

        let label = if let Token::Ident(_sym) = self.current_token() {
            None
        } else {
            None
        };

        self.expect(Token::Semicolon)?;

        Some(Stmt::Break(label))
    }

    /// Parse continue statement
    pub fn parse_continue_stmt(&mut self) -> Option<Stmt> {
        self.expect(Token::Continue)?;

        let label = None;

        self.expect(Token::Semicolon)?;

        Some(Stmt::Continue(label))
    }

    /// Parse block
    pub fn parse_block(&mut self) -> Option<Block> {
        let span_start = self.current_span();

        self.expect(Token::LBrace)?;

        let mut stmts = Vec::new();
        let mut trailing = None;

        while !self.is_at_end() && self.current_token() != Token::RBrace {
            if let Some(stmt) = self.parse_stmt() {
                if let Stmt::Expr(_) = stmt {
                    if self.current_token() == Token::RBrace || self.is_at_end() {
                        if let Stmt::Expr(expr) = stmt {
                            trailing = Some(Box::new(expr));
                        }
                        break;
                    }
                }
                stmts.push(stmt);
            } else {
                self.recover_to_stmt_sync();
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Block {
            stmts,
            trailing,
            span,
        })
    }

    /// Parse if expression
    pub fn parse_if_expr(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        self.expect(Token::If)?;

        let cond = self.parse_expr()?;
        let then_block = self.parse_block()?;

        let else_block = if self.match_token(Token::Else) {
            if self.match_token(Token::If) {
                let inner_if = self.parse_if_expr()?;
                Some(Box::new(inner_if))
            } else {
                let block = self.parse_block()?;
                Some(Box::new(Expr::Block(block)))
            }
        } else {
            None
        };

        Some(Expr::If(IfExpr {
            cond: Box::new(cond),
            then_block,
            else_block,
        }))
    }

    /// Parse match expression
    pub fn parse_match_expr(&mut self) -> Option<Expr> {
        let span_start = self.current_span();

        self.expect(Token::Match)?;

        let scrutinee = self.parse_expr()?;

        self.expect(Token::LBrace)?;

        let mut arms = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let pattern = self.parse_pattern()?;

            let guard = if self.match_token(Token::If) {
                self.parse_expr()
            } else {
                None
            };

            self.expect(Token::FatArrow)?;

            let body = self.parse_expr()?;

            arms.push(MatchArm {
                pattern,
                guard,
                body,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        Some(Expr::Match(MatchExpr {
            scrutinee: Box::new(scrutinee),
            arms,
        }))
    }

    /// Parse while expression (as expression form)
    pub fn parse_while_expr(&mut self) -> Option<Expr> {
        self.parse_while_stmt()?;
        None
    }

    /// Parse for expression
    pub fn parse_for_expr(&mut self) -> Option<Expr> {
        self.parse_for_stmt()?;
        None
    }

    /// Parse loop expression
    pub fn parse_loop_expr(&mut self) -> Option<Expr> {
        self.expect(Token::Loop)?;
        let body = self.parse_block()?;
        Some(Expr::Block(body))
    }

    /// Parse async expression
    pub fn parse_async_expr(&mut self) -> Option<Expr> {
        let _span_start = self.current_span();

        self.expect(Token::Async)?;

        let move_kw = self.match_token(Token::Mut);

        let body = self.parse_block()?;

        Some(Expr::Async(AsyncExpr { body, move_kw }))
    }
}
