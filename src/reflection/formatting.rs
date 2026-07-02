use std::rc::Rc;

use crate::core::parser::HasChildren;

pub(crate) fn format_iterable(parsers: &[Rc<dyn HasChildren>], offset: Option<usize>) -> String {
    let mut buffer = String::new();
    for (i, p) in parsers.iter().enumerate() {
        if 0 < i {
            buffer.push('\n');
        }
        match offset {
            Some(os) => buffer.push_str(&format!(" {}: ", os + i)),
            None => buffer.push_str(" - "),
        }
        buffer.push_str(&format!("{:?}", p));
    }
    buffer
}
