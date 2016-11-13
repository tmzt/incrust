
use std::fmt::Write;
use output_action::OutputAction;

pub trait WriteHtml {
    fn write_html(&self, w: &mut HtmlWrite);
}

pub trait HtmlWrite {
    fn write_html(&mut self, output_action: &OutputAction);
}

impl<T: Write> HtmlWrite for T {
    fn write_html(&mut self, output_action, &OutputAction) {
        match *output_action {
            OutputAction::Write(ref contents) => {
                write!(self, "{}", contents);
            },

            // For now, write the expression as a string
            OutputAction::WriteResult(ref template_expr) => {
                template_expr.write_html(self);
            },

            OutputAction::WriteOpen(ref element_type) => {
                write!(self, "<{}>", element_type);
            },

            OutputAction::WriteClose(ref element_type) => {
                write!(self, "</{}>", element_type);
            },

            OutputAction::WriteVoid(ref element_type) => {
                write!(self, "<{} />", element_type);
            }
        }
    }
}
