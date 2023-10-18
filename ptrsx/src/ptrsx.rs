use std::collections::BTreeMap;

#[derive(Default, Clone)]
pub struct Module {
    pub start: usize,
    pub end: usize,
    pub name: String,
}

#[derive(Default)]
pub struct PtrsxScanner {
    pub modules: Vec<Module>,
    pub forward: BTreeMap<usize, usize>,
    pub reverse: BTreeMap<usize, Vec<usize>>,
}
