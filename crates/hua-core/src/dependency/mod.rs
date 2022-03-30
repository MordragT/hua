use self::error::DependencyResult;
use crate::{Component, Package, Store};
use daggy::{Dag, NodeIndex};
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    hash::Hash,
};

mod error;
mod requirement;

pub use error::DependencyError;
pub use requirement::Requirement;

// TODO: Tests

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Conflict<'a> {
    Name(&'a String),
    Component(&'a Component),
}

#[derive(Debug)]
pub enum Step<'a> {
    Resolved(usize),
    Choice(HashSet<usize>),
    Unresolved(&'a Requirement),
}

impl<'a> Step<'a> {
    pub unsafe fn get_choice_unchecked(&self) -> &HashSet<usize> {
        match &self {
            Self::Choice(set) => set,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct DependencyGraph<'a> {
    relations: Dag<Step<'a>, &'a Requirement, usize>,
    names: HashSet<&'a String>,
    components: HashSet<&'a Component>,
}

impl<'a> DependencyGraph<'a> {
    pub fn new() -> Self {
        Self {
            relations: Dag::new(),
            names: HashSet::new(),
            components: HashSet::new(),
        }
    }

    fn inner_resolve_requirement(
        &mut self,
        req: &'a Requirement,
        store: &'a Store,
        choices: &mut HashMap<&'a Requirement, NodeIndex<usize>>,
    ) -> DependencyResult<NodeIndex<usize>> {
        let options = store
            .matches(req)
            .map(|package| {
                let index = unsafe { store.get_unchecked_index_of(package) };
                (index, package)
            })
            .collect::<HashMap<usize, &Package>>();

        let node = match options.len() {
            0 => self.relations.add_node(Step::Unresolved(req)),
            1 => {
                let (index, package) = unsafe { options.into_iter().next().unwrap_unchecked() };
                if let Some(conflict) = self.conflicts(package) {
                    return Err(conflict)?;
                }

                let node = self.relations.add_node(Step::Resolved(index));
                let children = self.inner_resolve(package.requires(), store, choices)?;
                let edges = children.into_iter().map(|(req, child)| (node, child, req));
                self.relations
                    .add_edges(edges)
                    .map_err(|err| DependencyError::CycleDetected {
                        error: err.to_string(),
                    })?;

                node
            }
            _len => {
                let node = self
                    .relations
                    .add_node(Step::Choice(options.keys().map(|key| *key).collect()));
                choices.insert(req, node);
                node
            }
        };
        Ok(node)
    }

    fn inner_resolve(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store,
        choices: &mut HashMap<&'a Requirement, NodeIndex<usize>>,
    ) -> DependencyResult<HashMap<&'a Requirement, NodeIndex<usize>>> {
        let mut nodes = HashMap::new();

        for req in requirements {
            let node = self.inner_resolve_requirement(req, store, choices)?;
            nodes.insert(req, node);
        }

        Ok(nodes)
    }

    fn inner_resolve_choices(
        &mut self,
        choices: HashMap<&'a Requirement, NodeIndex<usize>>,
        store: &'a Store,
    ) -> DependencyResult<()> {
        let mut future_choices = HashMap::new();
        for (req, node) in choices {
            let step = unsafe { self.relations.node_weight_mut(node).unwrap_unchecked() };

            let options = unsafe { step.get_choice_unchecked() }.clone();
            let mut result = None;

            for index in options.iter() {
                let package = unsafe { store.get_unchecked(*index) };
                if conflicts(&mut self.names, &mut self.components, &package).is_none() {
                    *step = Step::Resolved(*index);
                    result = Some(package);
                    break;
                }
            }

            match result {
                None => *step = Step::Unresolved(req),
                Some(package) => {
                    let children =
                        self.inner_resolve(package.requires(), store, &mut future_choices)?;
                    let edges = children.into_iter().map(|(req, child)| (node, child, req));
                    self.relations.add_edges(edges).map_err(|err| {
                        DependencyError::CycleDetected {
                            error: err.to_string(),
                        }
                    })?;
                }
            }
        }

        if future_choices.len() == 0 {
            Ok(())
        } else {
            self.inner_resolve_choices(future_choices, store)
        }
    }

    fn conflicts(&mut self, package: &'a Package) -> Option<Conflict> {
        conflicts(&mut self.names, &mut self.components, package)
    }

    pub fn resolve(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store,
    ) -> DependencyResult<()> {
        let to_resolve: Vec<&Requirement> = requirements.into_iter().collect();
        let mut choices = HashMap::new();

        // Check if top level requirements have conflicts with each other
        for req in to_resolve.iter() {
            for other in to_resolve.iter() {
                if req != other {
                    if !req.components().is_disjoint(other.components()) {
                        return Err(DependencyError::ConflictingComponents);
                    }
                    if req.name() == other.name() {
                        return Err(DependencyError::ConflictingNames);
                    }
                }
            }
        }

        let _nodes = self.inner_resolve(to_resolve, store, &mut choices)?;
        self.inner_resolve_choices(choices, store)?;

        Ok(())
    }
}

fn conflicts<'a>(
    names: &mut HashSet<&'a String>,
    components: &mut HashSet<&'a Component>,
    package: &'a Package,
) -> Option<Conflict<'a>> {
    if !names.insert(package.name()) {
        return Some(Conflict::Name(package.name()));
    }

    for component in package.provides() {
        if !components.insert(component) {
            return Some(Conflict::Component(component));
        }
    }
    None
}
