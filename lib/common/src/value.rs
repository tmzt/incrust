
use simple_expr::SimpleExpr;
use object_expr::ObjectExpr;


#[derive(Clone, Debug)]
pub enum Value {
    SimpleExprValue(SimpleExpr),
    ObjectExprValue(ObjectExpr),
}

pub mod parse {
    use super::Value;
    use syntax::ext::base::ExtCtxt;
    use syntax::parse::{token, PResult};
    use syntax::parse::parser::Parser;
    use simple_expr::parse::parse_simple_expr_until;
    use object_expr::parse::parse_object_expr;

    pub fn parse_value<'cx, 'a>(ecx: &'cx ExtCtxt<'a>, parser: &mut Parser<'a>) -> PResult<'a, Value> {
        parser.span_warn(parser.span, &format!("Parsing value - got token: {:?}", &parser.token));

        let sp = parser.span.clone();

        match parser.token {
            token::OpenDelim(token::Paren) => {
                try!(parser.expect(&token::OpenDelim(token::Paren)));
                parser.span_warn(parser.span, &format!("Parsing value - got open paren"));
                let simple_expr = try!(parse_simple_expr_until(ecx, parser, sp, &|token| token == &token::Semi || token == &token::CloseDelim(token::Brace)));
                Ok(Value::SimpleExprValue(simple_expr))
            },

            _ => {
                ecx.span_warn(parser.span, &format!("Parsing value - object expression"));
                //try!(parser.expect(&token::OpenDelim(token::Brace)));
                //ecx.span_warn(span, &format!("Parsing value - got open brace"));
                let object_expr = parse_object_expr(ecx, parser)?;
                Ok(Value::ObjectExprValue(object_expr))
            }
        }
    }
}

pub mod output {
    use super::Value;
    use common_write::WriteAs;
    use js_write::JsWriteExpr;

    impl<'a> WriteAs<'a, JsWriteExpr + 'a> for Value {
        fn write_to(&self, w: &mut JsWriteExpr) {
            w.write_value(self);
        }
    }
}
