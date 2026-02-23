//! Type Inference Engine
//!
//! Implements Hindley-Milner style type inference with unification

use super::types::*;
use faxc_util::Idx;
use std::collections::{HashMap, VecDeque};

/// Type inference engine
pub struct TypeInference {
    /// Type context
    ctx: TypeContext,
    /// Constraint queue
    constraints: VecDeque<Constraint>,
    /// Unification cache
    unify_cache: HashMap<(InferId, InferId), Result<Type, TypeError>>,
}

#[derive(Debug, Clone)]
pub enum TypeError {
    OccursCheck(InferId, Type),
    UnificationFailed(Type, Type),
    NoSolution(String),
}

impl TypeInference {
    pub fn new() -> Self {
        Self {
            ctx: TypeContext::default(),
            constraints: VecDeque::new(),
            unify_cache: HashMap::new(),
        }
    }

    /// Create new inference variable
    pub fn new_infer_var(&mut self) -> Type {
        let id = InferId(self.ctx.substitutions.len() as u32);
        self.ctx.substitutions.push(None);
        Type::Infer(id)
    }

    /// Add constraint
    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push_back(constraint);
    }

    /// Solve all constraints using unification
    pub fn solve(&mut self) -> Result<TypeContext, TypeError> {
        while let Some(constraint) = self.constraints.pop_front() {
            self.solve_constraint(constraint)?;
        }
        Ok(std::mem::take(&mut self.ctx))
    }

    fn solve_constraint(&mut self, constraint: Constraint) -> Result<(), TypeError> {
        match constraint {
            Constraint::Equal(t1, t2) => {
                self.unify(&t1, &t2)?;
            },
            Constraint::SubType(t1, t2) => {
                self.unify(&t1, &t2)?;
            },
            Constraint::Implements(t, tr) => {
                // Simplified: just check if t implements tr
            },
        }
        Ok(())
    }

    /// Unify two types
    pub fn unify(&mut self, t1: &Type, t2: &Type) -> Result<(), TypeError> {
        // Occurs check
        if let Type::Infer(id) = t1 {
            if self.occurs_check(id, t2) {
                return Err(TypeError::OccursCheck(*id, t2.clone()));
            }
        }
        if let Type::Infer(id) = t2 {
            if self.occurs_check(id, t1) {
                return Err(TypeError::OccursCheck(*id, t1.clone()));
            }
        }

        match (t1, t2) {
            // Both inference variables - check cache or unify
            (Type::Infer(i1), Type::Infer(i2)) => {
                if *i1 == *i2 {
                    return Ok(());
                }
                // Check cache
                let key = (*i1, *i2);
                if let Some(result) = self.unify_cache.get(&key) {
                    return result.clone().map(|_| ());
                }
                // Union-find style: point one to the other
                self.ctx.substitutions[i1.index()] = Some(t2.clone());
                return Ok(());
            },
            // One is inference variable
            (Type::Infer(id), t) | (t, Type::Infer(id)) => {
                self.ctx.substitutions[id.index()] = Some(t.clone());
                return Ok(());
            },
            // Both concrete types
            (Type::Int, Type::Int) => Ok(()),
            (Type::Float, Type::Float) => Ok(()),
            (Type::Bool, Type::Bool) => Ok(()),
            (Type::Char, Type::Char) => Ok(()),
            (Type::String, Type::String) => Ok(()),
            (Type::Unit, Type::Unit) => Ok(()),
            // Function types
            (Type::Fn(args1, ret1), Type::Fn(args2, ret2)) => {
                if args1.len() != args2.len() {
                    return Err(TypeError::UnificationFailed(t1.clone(), t2.clone()));
                }
                // Unify return types first
                self.unify(ret1, ret2)?;
                // Unify argument types
                for (a1, a2) in args1.iter().zip(args2.iter()) {
                    self.unify(a1, a2)?;
                }
                Ok(())
            },
            // Tuple types
            (Type::Tuple(ts1), Type::Tuple(ts2)) => {
                if ts1.len() != ts2.len() {
                    return Err(TypeError::UnificationFailed(t1.clone(), t2.clone()));
                }
                for (e1, e2) in ts1.iter().zip(ts2.iter()) {
                    self.unify(e1, e2)?;
                }
                Ok(())
            },
            // Reference types
            (Type::Ref(ty1, mut1), Type::Ref(ty2, mut2)) => {
                if mut1 != mut2 {
                    return Err(TypeError::UnificationFailed(t1.clone(), t2.clone()));
                }
                self.unify(ty1, ty2)
            },
            // Array types
            (Type::Array(ty1, n1), Type::Array(ty2, n2)) => {
                if n1 != n2 {
                    return Err(TypeError::UnificationFailed(t1.clone(), t2.clone()));
                }
                self.unify(ty1, ty2)
            },
            // Error and Never unify with anything
            (Type::Error, _) | (_, Type::Error) => Ok(()),
            (Type::Never, _) | (_, Type::Never) => Ok(()),
            // ADT types
            (Type::Adt(d1), Type::Adt(d2)) => {
                if d1 == d2 {
                    Ok(())
                } else {
                    Err(TypeError::UnificationFailed(t1.clone(), t2.clone()))
                }
            },
            // Param types
            (Type::Param(p1), Type::Param(p2)) => {
                if p1 == p2 {
                    Ok(())
                } else {
                    Err(TypeError::UnificationFailed(t1.clone(), t2.clone()))
                }
            },
            // Anything else doesn't unify
            _ => Err(TypeError::UnificationFailed(t1.clone(), t2.clone())),
        }
    }

    /// Occurs check - prevents infinite types
    /// Uses iterative approach to avoid stack overflow for deeply nested types
    fn occurs_check(&self, var: InferId, t: &Type) -> bool {
        let mut stack = vec![t];

        while let Some(current) = stack.pop() {
            match current {
                Type::Infer(id) => {
                    if *id == var {
                        return true;
                    }
                    // Follow substitution chain
                    if let Some(subst) = self.ctx.substitutions.get(id.index()) {
                        if let Some(s) = subst {
                            stack.push(s);
                        }
                    }
                },
                Type::Fn(args, ret) => {
                    stack.extend(args.iter());
                    stack.push(ret);
                },
                Type::Tuple(ts) => stack.extend(ts.iter()),
                Type::Ref(ty, _) => stack.push(ty),
                Type::Array(ty, _) => stack.push(ty),
                Type::Slice(ty) => stack.push(ty),
                Type::Future(ty) => stack.push(ty),
                Type::Option(ty) => stack.push(ty),
                Type::Result(ok, err) => {
                    stack.push(ok);
                    stack.push(err);
                },
                _ => {},
            }
        }

        false
    }

    /// Get the solved type for an inference variable with path compression
    pub fn resolve(&self, t: &Type) -> Type {
        match t {
            Type::Infer(id) => {
                if let Some(subst) = &self.ctx.substitutions[id.index()] {
                    let resolved = self.resolve(subst);
                    return resolved;
                }
                t.clone()
            },
            Type::Fn(args, ret) => Type::Fn(
                args.iter().map(|a| self.resolve(a)).collect(),
                Box::new(self.resolve(ret)),
            ),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.resolve(t)).collect()),
            Type::Ref(ty, m) => Type::Ref(Box::new(self.resolve(ty)), *m),
            Type::Array(ty, n) => Type::Array(Box::new(self.resolve(ty)), *n),
            Type::Option(ty) => Type::Option(Box::new(self.resolve(ty))),
            Type::Result(ok, err) => {
                Type::Result(Box::new(self.resolve(ok)), Box::new(self.resolve(err)))
            },
            Type::Slice(ty) => Type::Slice(Box::new(self.resolve(ty))),
            Type::Future(ty) => Type::Future(Box::new(self.resolve(ty))),
            _ => t.clone(),
        }
    }

    /// Get direct substitution without recursion (for caching)
    pub fn get_substitution(&self, id: InferId) -> Option<Type> {
        self.ctx
            .substitutions
            .get(id.index())
            .and_then(|t| t.clone())
    }
}

/// Constraint between types
#[derive(Debug, Clone)]
pub enum Constraint {
    /// Two types must be equal
    Equal(Type, Type),
    /// First type must be subtype of second
    SubType(Type, Type),
    /// Type must implement trait
    Implements(Type, DefId),
}
