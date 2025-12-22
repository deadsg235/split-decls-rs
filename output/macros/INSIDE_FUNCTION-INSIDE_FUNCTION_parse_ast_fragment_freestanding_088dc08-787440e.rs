INSIDE_FUNCTION ! { pub fn parse_ast_fragment <'a > (this : & mut Parser <'a >, kind : AstFragmentKind ,) -> PResult <'a , AstFragment > { Ok (match kind { AstFragmentKind :: Items => { let mut items = SmallVec :: new () ; while let Some (item) = this . parse_item (ForceCollect :: No) ? { items . push (item) ;}
AstFragment :: Items (items)}
AstFragmentKind :: TraitItems => { let mut items = SmallVec :: new () ; while let Some (item) = this . parse_trait_item (ForceCollect :: No) ? { items . extend (item) ;}
AstFragment :: TraitItems (items)}
AstFragmentKind :: ImplItems => { let mut items = SmallVec :: new () ; while let Some (item) = this . parse_impl_item (ForceCollect :: No) ? { items . extend (item) ;}
AstFragment :: ImplItems (items)}
AstFragmentKind :: TraitImplItems => { let mut items = SmallVec :: new () ; while let Some (item) = this . parse_impl_item (ForceCollect :: No) ? { items . extend (item) ;}
AstFragment :: TraitImplItems (items)}
AstFragmentKind :: ForeignItems => { let mut items = SmallVec :: new () ; while let Some (item) = this . parse_foreign_item (ForceCollect :: No) ? { items . extend (item) ;}
AstFragment :: ForeignItems (items)}
AstFragmentKind :: Stmts => { let mut stmts = SmallVec :: new () ; while this . token != token :: Eof && this . token != token :: CloseBrace { if let Some (stmt) = this . parse_full_stmt (AttemptLocalParseRecovery :: Yes) ? { stmts . push (stmt) ;}
} AstFragment :: Stmts (stmts)}
AstFragmentKind :: Expr => AstFragment :: Expr (this . parse_expr () ?) , AstFragmentKind :: MethodReceiverExpr => AstFragment :: MethodReceiverExpr (this . parse_expr () ?) , AstFragmentKind :: OptExpr => { if this . token != token :: Eof { AstFragment :: OptExpr (Some (this . parse_expr () ?))}
else { AstFragment :: OptExpr (None)}
} AstFragmentKind :: Ty => AstFragment :: Ty (this . parse_ty () ?) , AstFragmentKind :: Pat => AstFragment :: Pat (Box :: new (this . parse_pat_allow_top_guard (None , RecoverComma :: No , RecoverColon :: Yes , CommaRecoveryMode :: LikelyTuple ,) ?)) , AstFragmentKind :: Crate => AstFragment :: Crate (this . parse_crate_mod () ?) , AstFragmentKind :: Arms | AstFragmentKind :: ExprFields | AstFragmentKind :: PatFields | AstFragmentKind :: GenericParams | AstFragmentKind :: Params | AstFragmentKind :: FieldDefs | AstFragmentKind :: Variants | AstFragmentKind :: WherePredicates => panic ! ("unexpected AST fragment kind") , })}
}