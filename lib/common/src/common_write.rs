use std::fmt::Write;


pub trait WriteAs<'a, W: ?Sized + 'a> {
    fn write_to(&self, &'a mut W);
}
