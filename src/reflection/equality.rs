use std::{collections::HashSet, iter::zip, rc::Rc};

use crate::{
    core::{kind::PtrKey, parser::HasChildren},
    reflection::analyzer::ptr,
};

pub(crate) fn parsers_equal(p1: &Rc<dyn HasChildren>, p2: &Rc<dyn HasChildren>) -> bool {
    structurally_eq(p1.clone(), p2.clone(), &mut HashSet::new())
}

fn structurally_eq(
    p1: Rc<dyn HasChildren>,
    p2: Rc<dyn HasChildren>,
    seen: &mut HashSet<PtrKey>,
) -> bool {
    if Rc::ptr_eq(&p1, &p2) {
        return true;
    }
    if p1.kind() != p2.kind() {
        return false;
    }
    if !seen.insert(ptr(&p1)) {
        return true;
    }
    let (ac, bc) = (p1.children(), p2.children());
    ac.len() == bc.len() && zip(ac, bc).all(|(x, y)| structurally_eq(x, y, seen))
}

pub(crate) fn is_parser_iterable_equal(
    first: &[Rc<dyn HasChildren>],
    second: &[Rc<dyn HasChildren>],
) -> bool {
    first
        .iter()
        .all(|a| second.iter().any(|b| parsers_equal(a, b)))
        && second
            .iter()
            .all(|b| first.iter().any(|a| parsers_equal(a, b)))
}
