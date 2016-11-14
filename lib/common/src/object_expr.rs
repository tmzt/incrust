
use syntax::ast;
use simple_expr::SimpleExpr;


#[derive(Clone, Debug)]
pub enum ObjectExprAssignment {
    NewSimpleExprValue(ast::Name, SimpleExpr),
    // TODO: Make this explicitly name the dependent variable
    UpdateSimpleExprValue(ast::Name, SimpleExpr)
}

#[derive(Clone, Debug)]
pub struct ObjectExpr {
    cls_name: ast::Name,
    assignments: Vec<ObjectExprAssignment>
}

impl ObjectExpr {
    pub fn cls_name(&self) -> &ast::Name {
        &self.cls_name
    }

    pub fn assignments(&self) -> &[ObjectExprAssignment] {
        &self.assignments
    }
}

/*
#[derive(Clone, Debug)]
pub enum ObjectExprToken {
    ClsName(ast::Name),
    OpenBrace,
    CloseBrace,
    FieldAssignment(ObjectExprAssignment)
}
*/

pub mod parse {
    use syntax::ast::{self, LitKind, LitIntType, IntTy};
    use syntax::codemap::{Span, DUMMY_SP};
    use syntax::parse::token::DelimToken;
    use syntax::parse::token::BinOpToken as binops;
    use syntax::parse::token::Lit as literals;
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::common::SeqSep;
    use syntax::parse::parser::Parser;
    use simple_expr::{SimpleExpr, SimpleExprToken, SimpleExprNumber, SimpleExprWrite};
    use simple_expr::parse::parse_simple_expr_until;
    //use super::{ObjectExpr, ObjectExprToken};
    use super::{ObjectExpr, ObjectExprAssignment};

    fn parse_object_expr_assignment<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, ObjectExprAssignment> {
        let field_name = try!(parser.parse_ident()).to_string();
        // Colon
        try!(parser.expect(&token::Colon));
        // Open paren
        try!(parser.expect(&token::OpenDelim(token::Paren)));
        // Expression
        let simple_expr = {
            let sp = parser.span.clone();
            let expr_tts = parser.parse_seq_to_before_end(&token::CloseDelim(token::Paren), SeqSep::trailing_allowed(token::Semi), |pp| pp.parse_token_tree());
            let mut expr_parser = ecx.new_parser_from_tts(&expr_tts);
            try!(parse_simple_expr_until(ecx, &mut expr_parser, sp, &|token| token == &token::Eof))
        };
        // Close paren
        try!(parser.expect(&token::CloseDelim(token::Paren)));

        Ok(ObjectExprAssignment::UpdateSimpleExprValue(ecx.name_of(&field_name), simple_expr))
    }

    pub fn parse_object_expr<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, ObjectExpr> {
        //let mut assignments: Vec<ObjectExprAssignment> = vec![];

        // Expect an ident, which must begin with an uppercase letter
        let cls_name = {
            let cls_name = try!(parser.parse_ident()).to_string();
            if !cls_name.chars().nth(0).unwrap().is_uppercase() {
                ecx.span_err(parser.span, &format!("Expected ClsName (must begin with uppercase letter) got {}", cls_name));
            }

            ecx.name_of(&cls_name)
        };
        parser.span_warn(parser.span, &format!("Parsing object expression - got cls_name {:?}", &cls_name));

        let assignments = parser.parse_unspanned_seq(
            &token::OpenDelim(token::Brace),
            &token::CloseDelim(token::Brace),
            SeqSep::trailing_allowed(token::Comma),
            |p| parse_object_expr_assignment(ecx, p)
        )?;

        Ok(ObjectExpr {
            cls_name: cls_name,
            assignments: assignments
        })
    }
}

/*
pub mod output {
    use super::{ObjectExpr, ObjectExprAssignment};
    use common_write::WriteAs;
    use js_write::WriteJsExpr;

    impl WriteAs<WriteJsExpr> for ObjectExpr {
        fn write_to(&self, w: &mut JsWriteExpr) {
            let cls_name = self.cls_name();
            let assignments = self.assignments();

            for assignment in assignments {
                match assignment {
                    ObjectExprAssignment::NewSimpleExprValue(ref name, ref value) => {
                        w.write_value(value);
                    },

                    ObjectExprAssignment::UpdateSimpleExprValue(ref name, ref value) => {
                        w.write_value(value);
                    }
                }
            }
        }
    }
}
*/
