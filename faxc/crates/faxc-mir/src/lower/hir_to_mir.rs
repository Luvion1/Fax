//! HIR to MIR Lowering Implementation
//!
//! Transforms HIR (High-level IR) to MIR (Mid-level IR)

use crate::build::Builder;
use crate::mir::*;
use faxc_sem::hir;
use faxc_sem::Type;

pub fn lower_hir_function(hir_fn: &hir::FnItem) -> Function {
    let mut builder = Builder::new(hir_fn.name.clone(), hir_fn.ret_type.clone());

    let entry = builder.new_block();
    builder.set_current_block(entry);

    lower_expr(&mut builder, &hir_fn.body.value);

    builder.terminator(Terminator::Return);

    builder.build()
}

pub fn lower_expr(builder: &mut Builder, expr: &hir::Expr) -> Place {
    match expr {
        hir::Expr::Block {
            stmts,
            expr: trailing_expr,
            ..
        } => {
            for stmt in stmts {
                lower_stmt(builder, stmt);
            }
            if let Some(expr) = trailing_expr {
                lower_expr(builder, expr)
            } else {
                Place::Local(LocalId(0))
            }
        },

        hir::Expr::Literal { lit, ty } => {
            let constant = match lit {
                hir::Literal::Int(n) => ConstantKind::Int(*n),
                hir::Literal::Float(f) => ConstantKind::Float(*f),
                hir::Literal::String(s) => ConstantKind::String(*s),
                hir::Literal::Bool(b) => ConstantKind::Bool(*b),
                hir::Literal::Char(c) => ConstantKind::Int(*c as i64),
                hir::Literal::Unit => ConstantKind::Unit,
            };

            let temp = builder.add_local(ty.clone(), None);
            let place = Place::Local(temp);

            builder.assign(
                place.clone(),
                Rvalue::Use(Operand::Constant(Constant {
                    ty: ty.clone(),
                    kind: constant,
                })),
            );

            place
        },

        hir::Expr::Binary {
            op,
            left,
            right,
            ty,
        } => {
            let left_place = lower_expr(builder, left);
            let right_place = lower_expr(builder, right);

            let left_op = place_to_operand(left_place);
            let right_op = place_to_operand(right_place);

            let temp = builder.add_local(ty.clone(), None);
            let place = Place::Local(temp);

            builder.assign(
                place.clone(),
                Rvalue::BinaryOp(convert_binop(*op), Box::new(left_op), Box::new(right_op)),
            );

            place
        },

        hir::Expr::Var { def_id: _, ty: _ } => Place::Local(LocalId(0)),

        hir::Expr::If {
            cond,
            then_expr,
            else_expr,
            ty,
        } => {
            let cond_place = lower_expr(builder, cond);
            let cond_op = place_to_operand(cond_place);

            let then_block = builder.new_block();
            let else_block = builder.new_block();
            let join_block = builder.new_block();

            builder.terminator(Terminator::If {
                cond: cond_op,
                then_block,
                else_block,
            });

            builder.set_current_block(then_block);
            let _ = lower_expr(builder, then_expr);
            builder.terminator(Terminator::Goto { target: join_block });

            builder.set_current_block(else_block);
            if let Some(e) = else_expr {
                let _ = lower_expr(builder, e);
            }
            builder.terminator(Terminator::Goto { target: join_block });

            builder.set_current_block(join_block);
            let res_temp = builder.add_local(ty.clone(), None);
            Place::Local(res_temp)
        },

        hir::Expr::Call { func: _, args, ty } => {
            eprintln!("DEBUG: Handling call expression with ty={:?}", ty);
            let mut arg_operands = Vec::new();
            for arg in args {
                let place = lower_expr(builder, arg);
                arg_operands.push(place_to_operand(place));
            }

            let result_temp = builder.add_local(ty.clone(), None);

            builder.terminator(Terminator::Call {
                func: Operand::Constant(Constant {
                    ty: Type::Unit,
                    kind: ConstantKind::Int(0),
                }),
                args: arg_operands,
                destination: Place::Local(result_temp),
                target: None,
                cleanup: None,
            });

            Place::Local(result_temp)
        },

        _ => Place::Local(LocalId(0)),
    }
}

pub fn lower_stmt(builder: &mut Builder, stmt: &hir::Stmt) {
    match stmt {
        hir::Stmt::Let { pat, ty, init } => {
            let local = builder.add_local(ty.clone(), Some(pat.clone()));
            if let Some(init_expr) = init {
                let src_place = lower_expr(builder, init_expr);
                builder.assign(Place::Local(local), Rvalue::Use(Operand::Move(src_place)));
            }
        },
        hir::Stmt::Expr(expr) => {
            lower_expr(builder, expr);
        },
    }
}

fn place_to_operand(place: Place) -> Operand {
    Operand::Copy(place)
}

fn convert_binop(op: hir::BinOp) -> BinOp {
    match op {
        hir::BinOp::Add => BinOp::Add,
        hir::BinOp::Sub => BinOp::Sub,
        hir::BinOp::Mul => BinOp::Mul,
        hir::BinOp::Div => BinOp::Div,
        hir::BinOp::Mod => BinOp::Rem,
        hir::BinOp::Eq => BinOp::Eq,
        hir::BinOp::Ne => BinOp::Ne,
        hir::BinOp::Lt => BinOp::Lt,
        hir::BinOp::Gt => BinOp::Gt,
        hir::BinOp::Le => BinOp::Le,
        hir::BinOp::Ge => BinOp::Ge,
        hir::BinOp::And => BinOp::BitAnd,
        hir::BinOp::Or => BinOp::BitOr,
    }
}
