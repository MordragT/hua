use crate::{Component, Package, Store};
use daggy::{Dag, NodeIndex};
use std::collections::{HashMap, HashSet};

use super::{error::DependencyResult, Conflict, DependencyError, Requirement, Step};

#[derive(Debug)]
pub struct DependencyGraph<'a> {
    relations: Dag<Step<'a>, &'a Requirement, usize>,
    names: HashSet<&'a String>,
    components: HashSet<&'a Component>,
    visited: HashMap<&'a Requirement, NodeIndex<usize>>,
    inserted: HashMap<&'a Package, NodeIndex<usize>>,
}
impl<'a> DependencyGraph<'a> {
    pub fn new() -> Self {
        Self {
            relations: Dag::new(),
            names: HashSet::new(),
            components: HashSet::new(),
            visited: HashMap::new(),
            inserted: HashMap::new(),
        }
    }

    fn resolve_req(
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
                self.visited.insert(req, node);
                self.inserted.insert(package, node);

                let children = self.resolve_reqs(package.requires(), store, choices)?;
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
                    .add_node(Step::Choice(options.values().map(|p| *p).collect()));
                choices.insert(req, node);
                node
            }
        };
        Ok(node)
    }

    fn resolve_reqs(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store,
        choices: &mut HashMap<&'a Requirement, NodeIndex<usize>>,
    ) -> DependencyResult<HashMap<&'a Requirement, NodeIndex<usize>>> {
        let mut nodes = HashMap::new();

        for req in requirements {
            let node = if let Some(node) = self.visited.get(req) {
                *node
            } else {
                let inserted_options = self
                    .inserted
                    .iter()
                    .filter_map(|(package, index)| {
                        if package.matches(req) {
                            Some((*package, *index))
                        } else {
                            None
                        }
                    })
                    .collect::<HashMap<&Package, NodeIndex<usize>>>();

                match inserted_options.len() {
                    0 => self.resolve_req(req, store, choices)?,
                    1 => unsafe { inserted_options.into_iter().next().unwrap_unchecked() }.1,
                    _len => {
                        let node = self
                            .relations
                            .add_node(Step::Choice(inserted_options.keys().map(|p| *p).collect()));
                        choices.insert(req, node);
                        node
                    }
                }
            };
            nodes.insert(req, node);
        }

        Ok(nodes)
    }

    fn resolve_choices(
        &mut self,
        choices: HashMap<&'a Requirement, NodeIndex<usize>>,
        store: &'a Store,
    ) -> DependencyResult<()> {
        let mut future_choices = HashMap::new();
        for (req, node) in choices {
            let step = unsafe { self.relations.node_weight_mut(node).unwrap_unchecked() };

            let options = unsafe { step.as_choice_unchecked() }.clone();
            let mut result = None;

            for package in options.iter() {
                let index = unsafe { store.get_unchecked_index_of(package) };
                if conflicts(&mut self.names, &mut self.components, &package).is_none() {
                    *step = Step::Resolved(index);
                    result = Some(package);
                    break;
                }
            }

            match result {
                None => *step = Step::Unresolved(req),
                Some(package) => {
                    let children =
                        self.resolve_reqs(package.requires(), store, &mut future_choices)?;
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
            self.resolve_choices(future_choices, store)
        }
    }

    fn conflicts(&mut self, package: &'a Package) -> Option<Conflict> {
        conflicts(&mut self.names, &mut self.components, package)
    }

    pub fn unresolved_requirements(&self) -> impl Iterator<Item = &Requirement> {
        self.relations
            .graph()
            .node_weights()
            .filter_map(Step::as_unresolved)
    }

    pub fn is_resolved(&self) -> bool {
        self.relations.graph().node_weights().all(Step::is_resolved)
    }

    pub fn resolve(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store,
    ) -> DependencyResult<bool> {
        let to_resolve: Vec<&Requirement> = requirements.into_iter().collect();
        let mut choices = HashMap::new();

        let _nodes = self.resolve_reqs(to_resolve, store, &mut choices)?;
        self.resolve_choices(choices, store)?;

        Ok(self.is_resolved())
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

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;

    use crate::{dependency::DependencyError, DependencyGraph, Store};
    use crate::{support::*, Error};
    use temp_dir::TempDir;

    #[test]
    fn dependency_graph_resolve_ok_true() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::create_at_path(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [to_req(&one)]);

        let three_path = temp_dir.child("three");
        let three = pkg("three", &three_path);

        let four_path = temp_dir.child("four");
        let four = pkg_req(
            "four",
            &four_path,
            [to_req(&one), to_req(&two), to_req(&three)],
        );

        let req_four = to_req(&four);
        let req_one = to_req(&one);
        let reqs = vec![&req_four, &req_one];

        let pkgs = vec![
            (one, one_path),
            (two, two_path),
            (three, three_path),
            (four, four_path),
        ];
        store
            .extend(pkgs)
            .collect::<Result<Vec<usize>, Error>>()
            .unwrap();

        let is_resolved = graph.resolve(reqs, &store).unwrap();

        assert!(is_resolved);
        println!("Graph: {graph:?}");
    }

    #[test]
    fn dependency_graph_resolve_ok_false() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::create_at_path(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("other", ">1.0")]);

        let (one_req, two_req) = (to_req(&one), to_req(&two));
        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![(one, one_path), (two, two_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<usize>, Error>>()
            .unwrap();

        let is_resolved = graph.resolve(reqs, &store).unwrap();

        assert!(!is_resolved);
        println!("Graph: {graph:?}");
    }

    #[test]
    fn dependency_graph_resolve_err_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::create_at_path(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg_req("one", &one_path, [req("two", ">=1.0")]);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("one", ">=1.0")]);

        let (one_req, two_req) = (to_req(&one), to_req(&two));
        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![(one, one_path), (two, two_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<usize>, Error>>()
            .unwrap();

        let err = graph.resolve(reqs, &store).unwrap_err();

        assert_matches!(err, DependencyError::CycleDetected { error: _ });
    }

    #[test]
    fn depdendency_graph_resolve_err_conflict_name() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::create_at_path(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [to_req(&one)]);

        let three_path = temp_dir.child("three");
        let three = pkg_ver("one", &three_path, "1.1.1");

        let req_three = to_req(&three);
        let req_two = to_req(&two);
        let reqs = vec![&req_two, &req_three];

        let pkgs = vec![(one, one_path), (two, two_path), (three, three_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<usize>, Error>>()
            .unwrap();

        let err = graph.resolve(reqs, &store).unwrap_err();

        assert_eq!(
            err,
            DependencyError::ConflictingName {
                name: "one".to_owned()
            }
        );
    }

    #[test]
    fn depdendency_graph_resolve_err_conflict_component() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::create_at_path(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);

        let two_path = temp_dir.child("two");
        let two = pkg_prov("two", &two_path, "one");

        let req_one = to_req(&one);
        let req_two = to_req(&two);
        let reqs = vec![&req_two, &req_one];

        let pkgs = vec![(one, one_path), (two, two_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<usize>, Error>>()
            .unwrap();

        let err = graph.resolve(reqs, &store).unwrap_err();

        assert_matches!(err, DependencyError::ConflictingComponent { component: _ });
    }
}
