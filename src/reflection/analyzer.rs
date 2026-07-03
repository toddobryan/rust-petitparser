use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::{
    core::parser::HasChildren,
    parser::misc::epsilon::epsilon,
    reflection::path::{ParserPath, depth_first_search},
};

type PtrKey = *const ();

pub(crate) fn ptr(p: &Rc<dyn HasChildren>) -> PtrKey {
    Rc::as_ptr(p) as PtrKey
}

pub struct Analyzer {
    pub parsers: Vec<Rc<dyn HasChildren>>,
    sentinel: Rc<dyn HasChildren>, // an EpsilonParser<()> used as nullable marker
    first_sets: HashMap<PtrKey, HashSet<PtrKey>>,
    cycle_sets: HashMap<PtrKey, Vec<Rc<dyn HasChildren>>>,
    by_ptr: HashMap<PtrKey, Rc<dyn HasChildren>>, // ptr -> Rc, for lookup
}

impl Analyzer {
    pub fn new(root: Rc<dyn HasChildren>) -> Self {
        let mut stack: Vec<Rc<dyn HasChildren>> = vec![root.clone()];
        let mut seen: HashSet<PtrKey> = HashSet::from([ptr(&root)]);
        let mut parsers: Vec<Rc<dyn HasChildren>> = Vec::new();
        let mut by_ptr: HashMap<PtrKey, Rc<dyn HasChildren>> = HashMap::new();

        while let Some(current) = stack.pop() {
            parsers.push(current.clone());
            by_ptr.insert(ptr(&current), current.clone());
            for child in current.children().into_iter().rev() {
                if seen.insert(ptr(&child)) {
                    stack.push(child);
                }
            }
        }

        let sentinel: Rc<dyn HasChildren> = Rc::new(epsilon());
        by_ptr.insert(ptr(&sentinel), sentinel.clone());

        let mut analyzer = Analyzer {
            parsers,
            sentinel,
            first_sets: HashMap::new(),
            cycle_sets: HashMap::new(),
            by_ptr,
        };

        analyzer.compute_first_sets();
        analyzer.compute_cycle_sets();
        analyzer
    }

    pub fn all_children(&self, parser: &Rc<dyn HasChildren>) -> Vec<Rc<dyn HasChildren>> {
        let mut reachable: HashSet<PtrKey> = HashSet::new();
        let mut result: Vec<Rc<dyn HasChildren>> = Vec::new();
        let mut stack: Vec<Rc<dyn HasChildren>> = parser.children(); // start from children, not self
        while let Some(current) = stack.pop() {
            if reachable.insert(ptr(&current)) {
                result.push(current.clone());
                for child in current.children() {
                    if !reachable.contains(&ptr(&child)) {
                        stack.push(child);
                    }
                }
            }
        }
        result
    }

    pub fn find_path(
        &self,
        source: &Rc<dyn HasChildren>,
        predicate: &impl Fn(&ParserPath) -> bool,
    ) -> Option<ParserPath> {
        let mut path: Option<ParserPath> = None;
        for current in self.find_all_paths(source, predicate) {
            if path.is_none() || current.len() < path.as_ref()?.len() {
                path = Some(current);
            }
        }
        path
    }

    pub fn find_all_paths(
        &self,
        source: &Rc<dyn HasChildren>,
        predicate: &impl Fn(&ParserPath) -> bool,
    ) -> Vec<ParserPath> {
        assert!(
            self.by_ptr.contains_key(&ptr(source)),
            "source is not part of this analyzer"
        );
        depth_first_search(
            &mut ParserPath::new(vec![source.clone()], Vec::new()),
            predicate,
        )
    }

    pub fn first_set(&self, p: &Rc<dyn HasChildren>) -> Vec<Rc<dyn HasChildren>> {
        let sentinel_ptr = ptr(&self.sentinel);
        self.first_sets[&ptr(p)]
            .iter()
            .filter(|&&k| k != sentinel_ptr)
            .filter_map(|k| self.by_ptr.get(k).cloned())
            .collect()
    }

    pub fn is_nullable(&self, p: &Rc<dyn HasChildren>) -> bool {
        self.first_sets[&ptr(p)].contains(&ptr(&self.sentinel))
    }

    pub fn cycle_set(&self, p: &Rc<dyn HasChildren>) -> &[Rc<dyn HasChildren>] {
        self.cycle_sets
            .get(&ptr(p))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    fn compute_first_sets(&mut self) {
        let mut first_sets: HashMap<PtrKey, HashSet<PtrKey>> = HashMap::new();
        for p in self.parsers.clone() {
            let mut p_first_set: HashSet<PtrKey> = HashSet::new();
            if p.children().is_empty() {
                p_first_set.insert(ptr(&p));
            }
            if p.is_directly_nullable() {
                p_first_set.insert(ptr(&self.sentinel));
            }
            first_sets.insert(ptr(&p), p_first_set);
        }
        loop {
            let mut changed: bool = false;
            for p in self.parsers.clone() {
                changed |= Self::expand_first_set(&p, &mut first_sets, ptr(&self.sentinel));
            }
            if !changed {
                break;
            }
        }
        self.first_sets = first_sets;
    }

    fn expand_first_set(
        parser: &Rc<dyn HasChildren>,
        first_sets: &mut HashMap<PtrKey, HashSet<PtrKey>>,
        sentinel_ptr: PtrKey,
    ) -> bool {
        let mut changed: bool = false;
        if parser.is_sequence() {
            for child in parser.children() {
                let mut is_nullable = false;
                let to_add: Vec<PtrKey> = first_sets[&ptr(&child)].iter().copied().collect();
                let first_set = first_sets.get_mut(&ptr(parser)).unwrap();
                for pk in to_add {
                    if pk == sentinel_ptr {
                        is_nullable = true;
                    } else {
                        changed |= first_set.insert(pk);
                    }
                }
                if !is_nullable {
                    return changed;
                }
            }
            changed |= first_sets
                .get_mut(&ptr(parser))
                .unwrap()
                .insert(sentinel_ptr);
        } else {
            for child in parser.children() {
                let to_add: Vec<PtrKey> = first_sets[&ptr(&child)].iter().copied().collect();
                let first_set = first_sets.get_mut(&ptr(parser)).unwrap();
                for pk in to_add {
                    changed |= first_set.insert(pk);
                }
            }
        }
        changed
    }

    fn compute_cycle_sets(&mut self) {
        let mut cycle_sets: HashMap<PtrKey, Vec<Rc<dyn HasChildren>>> = HashMap::new();
        for p in self.parsers.clone() {
            self.compute_cycle_set(&p, &mut cycle_sets, &mut Vec::new());
        }
        self.cycle_sets = cycle_sets;
    }

    fn compute_cycle_set(
        &mut self,
        parser: &Rc<dyn HasChildren>,
        cycle_sets: &mut HashMap<PtrKey, Vec<Rc<dyn HasChildren>>>,
        stack: &mut Vec<Rc<dyn HasChildren>>,
    ) {
        let p_ptr = ptr(parser);

        if cycle_sets.contains_key(&p_ptr) {
            return;
        }
        if parser.children().is_empty() {
            cycle_sets.insert(p_ptr, Vec::new());
            return;
        }
        let children = &self.compute_cycle_children(parser);
        for child in children {
            let index = stack.iter().position(|p| ptr(p) == ptr(child));
            match index {
                Some(index) => {
                    let cycle = &stack[index..];
                    for p in cycle {
                        cycle_sets.insert(ptr(p), cycle.to_vec());
                    }
                    return;
                }
                None => {
                    stack.push(child.clone());
                    self.compute_cycle_set(child, cycle_sets, stack);
                    stack.pop();
                }
            }
        }
        if !cycle_sets.contains_key(&ptr(parser)) {
            cycle_sets.insert(p_ptr, Vec::new());
        }
    }

    fn compute_cycle_children(&self, parser: &Rc<dyn HasChildren>) -> Vec<Rc<dyn HasChildren>> {
        if parser.is_sequence() {
            let mut children: Vec<Rc<dyn HasChildren>> = Vec::new();
            for child in parser.children() {
                children.push(child.clone());
                if !self.first_sets[&ptr(&child)].contains(&ptr(&self.sentinel)) {
                    break;
                }
            }
            children
        } else {
            parser.children()
        }
    }
}
