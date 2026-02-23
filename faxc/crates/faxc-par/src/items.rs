//! Item parsing - top-level declarations (fn, struct, enum, trait, impl, use, etc.)

use crate::ast::*;
use crate::Parser;
use faxc_lex::Token;

impl<'a> Parser<'a> {
    /// Parse a single top-level item
    pub fn parse_item(&mut self) -> Option<Item> {
        let visibility = self.parse_visibility();
        let async_kw = self.match_token(Token::Async);

        match self.current_token() {
            Token::Fn => self.parse_fn_item(visibility, async_kw),
            Token::Struct => self.parse_struct_item(visibility),
            Token::Enum => self.parse_enum_item(visibility),
            Token::Trait => self.parse_trait_item(visibility),
            Token::Impl => self.parse_impl_item(),
            Token::Use => self.parse_use_item(),
            Token::Mod => self.parse_mod_item(visibility),
            Token::Const => self.parse_const_item(visibility),
            Token::Static => self.parse_static_item(visibility),
            _ => {
                self.error(
                    "expected item: fn, struct, enum, trait, impl, use, mod, const, or static",
                );
                None
            },
        }
    }

    /// Parse visibility modifier
    pub fn parse_visibility(&mut self) -> Visibility {
        if !self.match_token(Token::Pub) {
            return Visibility::Private;
        }

        if self.match_token(Token::LParen) {
            let vis = match self.current_token() {
                Token::Crate => {
                    self.advance();
                    Visibility::Crate
                },
                Token::Super => {
                    self.advance();
                    Visibility::Super
                },
                Token::Ident(sym) if sym.as_str() == "in" => {
                    self.advance();
                    let path = self.parse_path();
                    Visibility::Restricted(path)
                },
                _ => {
                    self.error("expected 'crate', 'super', or 'in' in visibility");
                    Visibility::Public
                },
            };
            self.expect(Token::RParen);
            vis
        } else {
            Visibility::Public
        }
    }

    /// Parse function item
    pub fn parse_fn_item(&mut self, visibility: Visibility, async_kw: bool) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Fn)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let params = self.parse_params()?;
        let where_clause = self.parse_where_clause();
        let ret_type = self.parse_return_type();
        let body = self.parse_block()?;

        let span = self.span_from_start(span_start);

        Some(Item::Fn(FnItem {
            name,
            generics,
            params,
            ret_type,
            body,
            visibility,
            span,
            async_kw,
            where_clause,
        }))
    }

    /// Parse generic parameters
    pub fn parse_generics(&mut self) -> Vec<GenericParam> {
        if !self.match_token(Token::Lt) {
            return Vec::new();
        }

        let mut params = Vec::new();

        while !self.is_at_end() && self.current_token() != Token::Gt {
            let name = match self.parse_ident() {
                Some(n) => n,
                None => break,
            };

            let mut bounds = Vec::new();
            if self.match_token(Token::Colon) {
                loop {
                    if let Some(ty) = self.parse_type() {
                        bounds.push(ty);
                    }
                    if !self.match_token(Token::Plus) {
                        break;
                    }
                }
            }

            params.push(GenericParam { name, bounds });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::Gt);
        params
    }

    /// Parse where clause
    pub fn parse_where_clause(&mut self) -> Option<WhereClause> {
        if !self.match_token(Token::Where) {
            return None;
        }

        let mut bounds = Vec::new();

        loop {
            let ty = self.parse_type()?;
            self.expect(Token::Colon)?;

            let mut traits = Vec::new();
            loop {
                let path = self.parse_path();
                traits.push(path);
                if !self.match_token(Token::Plus) {
                    break;
                }
            }

            bounds.push(WhereBound { ty, traits });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        Some(WhereClause { bounds })
    }

    /// Parse function parameters
    pub fn parse_params(&mut self) -> Option<Vec<Param>> {
        self.expect(Token::LParen)?;

        let mut params = Vec::new();

        if !self.match_token(Token::RParen) {
            loop {
                let mutable = self.match_token(Token::Mut);
                let name = self.parse_ident()?;
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;

                params.push(Param { name, ty, mutable });

                if !self.match_token(Token::Comma) {
                    break;
                }
            }
            self.expect(Token::RParen)?;
        }

        Some(params)
    }

    /// Parse return type
    pub fn parse_return_type(&mut self) -> Option<Type> {
        if !self.match_token(Token::Arrow) {
            return None;
        }
        self.parse_type()
    }

    /// Parse struct item
    pub fn parse_struct_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Struct)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        self.expect(Token::LBrace)?;

        let mut fields = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let field_vis = self.parse_visibility();
            let field_name = self.parse_ident()?;
            self.expect(Token::Colon)?;
            let field_ty = self.parse_type()?;

            fields.push(Field {
                name: field_name,
                ty: field_ty,
                visibility: field_vis,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Item::Struct(StructItem {
            name,
            generics,
            fields,
            visibility,
            span,
            where_clause,
        }))
    }

    /// Parse enum item
    pub fn parse_enum_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Enum)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        self.expect(Token::LBrace)?;

        let mut variants = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            let variant_name = self.parse_ident()?;

            let data = if self.match_token(Token::LParen) {
                let mut types = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RParen {
                    if let Some(ty) = self.parse_type() {
                        types.push(ty);
                    }
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RParen);
                VariantData::Tuple(types)
            } else if self.match_token(Token::LBrace) {
                let mut fields = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RBrace {
                    let field_name = self.parse_ident()?;
                    self.expect(Token::Colon)?;
                    let field_ty = self.parse_type()?;
                    fields.push(Field {
                        name: field_name,
                        ty: field_ty,
                        visibility: Visibility::Private,
                    });
                    if !self.match_token(Token::Comma) {
                        break;
                    }
                }
                self.expect(Token::RBrace);
                VariantData::Struct(fields)
            } else {
                VariantData::Unit
            };

            variants.push(Variant {
                name: variant_name,
                data,
            });

            if !self.match_token(Token::Comma) {
                break;
            }
        }

        self.expect(Token::RBrace)?;

        let span = self.span_from_start(span_start);

        Some(Item::Enum(EnumItem {
            name,
            generics,
            variants,
            visibility,
            span,
            where_clause,
        }))
    }

    /// Parse trait item
    pub fn parse_trait_item(&mut self, visibility: Visibility) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Trait)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();

        let mut supertraits = Vec::new();
        if self.match_token(Token::Colon) {
            loop {
                if let Some(ty) = self.parse_type() {
                    supertraits.push(ty);
                }
                if !self.match_token(Token::Plus) {
                    break;
                }
            }
        }

        self.expect(Token::LBrace)?;

        let mut items = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            if self.current_token() == Token::Fn {
                if let Some(sig) = self.parse_fn_sig() {
                    items.push(TraitMember::Method(sig));
                }
            } else {
                self.recover_to_stmt_sync();
            }

            if self.match_token(Token::Semicolon) {
                continue;
            }
        }

        self.expect(Token::RBrace)?;

        Some(Item::Trait(TraitItem {
            name,
            generics,
            items,
            supertraits,
            visibility,
        }))
    }

    /// Parse function signature (for traits)
    pub fn parse_fn_sig(&mut self) -> Option<FnSig> {
        self.expect(Token::Fn)?;

        let name = self.parse_ident()?;
        let generics = self.parse_generics();
        let params = self.parse_params()?;
        let ret_type = self.parse_return_type();

        self.match_token(Token::Semicolon);

        Some(FnSig {
            name,
            generics,
            params,
            ret_type,
        })
    }

    /// Parse impl item
    pub fn parse_impl_item(&mut self) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Impl)?;

        let generics = self.parse_generics();
        let where_clause = self.parse_where_clause();

        let trait_ref = if self.current_token() != Token::For {
            let ty = self.parse_type()?;
            if self.match_token(Token::For) {
                Some(ty)
            } else {
                let self_ty = ty;
                self.expect(Token::LBrace)?;

                let mut items = Vec::new();
                while !self.is_at_end() && self.current_token() != Token::RBrace {
                    if self.current_token() == Token::Fn {
                        if let Some(item) = self.parse_item() {
                            if let Item::Fn(fn_item) = item {
                                items.push(ImplMember::Method(fn_item));
                            }
                        }
                    } else {
                        self.recover_to_stmt_sync();
                    }
                }
                self.expect(Token::RBrace)?;

                return Some(Item::Impl(ImplItem {
                    generics,
                    trait_ref: None,
                    self_ty,
                    items,
                    where_clause,
                }));
            }
        } else {
            None
        };

        self.expect(Token::For)?;
        let self_ty = self.parse_type()?;

        self.expect(Token::LBrace)?;

        let mut items = Vec::new();
        while !self.is_at_end() && self.current_token() != Token::RBrace {
            if self.current_token() == Token::Fn {
                if let Some(item) = self.parse_item() {
                    if let Item::Fn(fn_item) = item {
                        items.push(ImplMember::Method(fn_item));
                    }
                }
            } else {
                self.recover_to_stmt_sync();
            }
        }
        self.expect(Token::RBrace)?;

        Some(Item::Impl(ImplItem {
            generics,
            trait_ref,
            self_ty,
            items,
            where_clause,
        }))
    }

    /// Parse use item
    pub fn parse_use_item(&mut self) -> Option<Item> {
        let _span_start = self.current_span();

        self.expect(Token::Use)?;

        let path = self.parse_path();
        let mut alias = None;
        let mut is_glob = false;

        if self.match_token(Token::As) {
            alias = self.parse_ident();
        }

        if self.match_token(Token::Star) {
            is_glob = true;
        }

        self.expect(Token::Semicolon)?;

        Some(Item::Use(UseItem {
            path,
            alias,
            is_glob,
        }))
    }

    /// Parse mod item
    pub fn parse_mod_item(&mut self, _visibility: Visibility) -> Option<Item> {
        self.expect(Token::Mod)?;
        let _name = self.parse_ident()?;

        if self.match_token(Token::Semicolon) {
            return None;
        }

        if self.match_token(Token::LBrace) {
            let mut items = Vec::new();
            while !self.is_at_end() && self.current_token() != Token::RBrace {
                if let Some(item) = self.parse_item() {
                    items.push(item);
                } else {
                    self.recover_to_sync_point();
                }
            }
            self.expect(Token::RBrace)?;
        }

        None
    }

    /// Parse const item
    pub fn parse_const_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Const)?;
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = self.span_from_start(span_start);

        Some(Item::Const(ConstItem {
            name,
            ty,
            value,
            visibility,
            span,
        }))
    }

    /// Parse static item
    pub fn parse_static_item(&mut self, visibility: Visibility) -> Option<Item> {
        let span_start = self.current_span();

        self.expect(Token::Static)?;
        let mutable = self.match_token(Token::Mut);
        let name = self.parse_ident()?;
        self.expect(Token::Colon)?;
        let ty = self.parse_type()?;
        self.expect(Token::Eq)?;
        let value = self.parse_expr()?;
        self.expect(Token::Semicolon)?;

        let span = self.span_from_start(span_start);

        Some(Item::Static(StaticItem {
            name,
            ty,
            value,
            mutable,
            visibility,
            span,
        }))
    }
}
