use crate::{
    store::{Backend, Blob, PackageDesc, PackageId},
    Store,
};
use daggy::{Dag, NodeIndex};
use std::collections::{HashMap, HashSet};

use super::{Conflict, DependencyError, DependencyResult, Requirement, Step};

#[derive(Debug)]
pub struct DependencyGraph<'a> {
    relations: Dag<Step<'a>, &'a Requirement, usize>,
    names: HashSet<&'a String>,
    objects: HashSet<&'a Blob>,
    visited: HashMap<&'a Requirement, NodeIndex<usize>>,
    inserted: HashMap<PackageId, NodeIndex<usize>>,
}

// TODO add resolve_packages function that resolves the graph
// for specific packages not just a requirement which could lead to
// other packages to be installed then it was intended

impl<'a> DependencyGraph<'a> {
    pub fn new() -> Self {
        Self {
            relations: Dag::new(),
            names: HashSet::new(),
            objects: HashSet::new(),
            visited: HashMap::new(),
            inserted: HashMap::new(),
        }
    }

    fn resolve_req<B: Backend>(
        &mut self,
        req: &'a Requirement,
        store: &'a Store<B>,
        choices: &mut HashMap<&'a Requirement, NodeIndex<usize>>,
    ) -> DependencyResult<NodeIndex<usize>> {
        let options = store
            .packages()
            .matches(req)
            .collect::<HashMap<&PackageId, &PackageDesc>>();

        let node = match options.len() {
            0 => self.relations.add_node(Step::Unresolved(req)),
            1 => {
                let (id, package) = unsafe { options.into_iter().next().unwrap_unchecked() };
                if let Some(conflict) = self.conflicts(package) {
                    return Err(conflict)?;
                }

                let node = self.relations.add_node(Step::Resolved(*id));
                self.visited.insert(req, node);
                self.inserted.insert(*id, node);

                let children = self.resolve_reqs(&package.requires, store, choices)?;
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
                    .add_node(Step::Choice(options.keys().map(|id| **id).collect()));
                choices.insert(req, node);
                node
            }
        };
        Ok(node)
    }

    fn resolve_reqs<B: Backend>(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store<B>,
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
                    .filter_map(|(id, index)| {
                        let package = unsafe { store.packages().get_unchecked(id) };
                        if package.matches(req) {
                            Some((*id, *index))
                        } else {
                            None
                        }
                    })
                    .collect::<HashMap<PackageId, NodeIndex<usize>>>();

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

    fn resolve_choices<B: Backend>(
        &mut self,
        choices: HashMap<&'a Requirement, NodeIndex<usize>>,
        store: &'a Store<B>,
    ) -> DependencyResult<()> {
        let mut future_choices = HashMap::new();
        for (req, node) in choices {
            let step = unsafe { self.relations.node_weight_mut(node).unwrap_unchecked() };

            let options = unsafe { step.as_choice_unchecked() }.clone();
            let mut result = None;

            for id in options.iter() {
                let package = unsafe { store.packages().get_unchecked(&id) };
                if conflicts(&mut self.names, &mut self.objects, &package).is_none() {
                    *step = Step::Resolved(*id);
                    result = Some(package);
                    break;
                }
            }

            match result {
                None => *step = Step::Unresolved(req),
                Some(package) => {
                    let children =
                        self.resolve_reqs(&package.requires, store, &mut future_choices)?;
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

    fn conflicts(&mut self, package: &'a PackageDesc) -> Option<Conflict> {
        conflicts(&mut self.names, &mut self.objects, package)
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

    pub fn resolve<B: Backend>(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store<B>,
    ) -> DependencyResult<impl Iterator<Item = PackageId> + '_> {
        let mut choices = HashMap::new();

        let _nodes = self.resolve_reqs(requirements, store, &mut choices)?;
        self.resolve_choices(choices, store)?;

        if self.is_resolved() {
            let iter = self
                .relations
                .graph()
                .node_weights()
                .map(|node| unsafe { node.as_resolved_unchecked() });
            Ok(iter)
        } else {
            Err(DependencyError::RequirementsNotResolvable)
        }
    }

    pub fn nodes(&self) -> impl Iterator<Item = &Step> {
        self.relations.graph().node_weights()
    }
}

fn conflicts<'a>(
    names: &mut HashSet<&'a String>,
    objects: &mut HashSet<&'a Blob>,
    package: &'a PackageDesc,
) -> Option<Conflict<'a>> {
    if !names.insert(&package.name) {
        return Some(Conflict::Name(&package.name));
    }

    for blob in package.blobs.iter() {
        if !objects.insert(blob) {
            return Some(Conflict::Blob(blob));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use crate::store::{LocalBackend, StoreError};
    use crate::support::*;
    use crate::{dependency::DependencyError, DependencyGraph, Store};
    use std::assert_matches::assert_matches;
    use temp_dir::TempDir;

    #[test]
    fn dependency_graph_resolve_ok_true() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();

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
            .collect::<Result<Vec<bool>, StoreError>>()
            .unwrap();

        let indices = graph.resolve(reqs, &store).unwrap();

        assert_eq!(4, indices.count());
        println!("Graph: {graph:?}");
    }

    #[test]
    fn dependency_graph_resolve_err_not_resolvable() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("other", ">1.0")]);

        let (one_req, two_req) = (to_req(&one), to_req(&two));
        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![(one, one_path), (two, two_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<bool>, StoreError>>()
            .unwrap();

        let err = graph
            .resolve(reqs, &store)
            .map(|iter| iter.count())
            .unwrap_err();

        assert_matches!(err, DependencyError::RequirementsNotResolvable)
    }

    #[test]
    fn dependency_graph_resolve_err_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg_req("one", &one_path, [req("two", ">=1.0")]);

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("one", ">=1.0")]);

        let (one_req, two_req) = (to_req(&one), to_req(&two));
        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![(one, one_path), (two, two_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<bool>, StoreError>>()
            .unwrap();

        let err = graph
            .resolve(reqs, &store)
            .map(|iter| iter.count())
            .unwrap_err();

        assert_matches!(err, DependencyError::CycleDetected { error: _ });
    }

    #[test]
    fn depdendency_graph_resolve_err_conflict_name() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let req_one = req("one", ">=1.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req_one]);

        let three_path = temp_dir.child("three");
        let three = pkg_prov("one", &three_path, "other");

        let req_three = to_req(&three);
        let req_two = to_req(&two);
        let reqs = vec![&req_two, &req_three];

        let pkgs = vec![(one, one_path), (two, two_path), (three, three_path)];
        store
            .extend(pkgs)
            .collect::<Result<Vec<bool>, StoreError>>()
            .unwrap();

        let err = graph
            .resolve(reqs, &store)
            .map(|iter| iter.count())
            .unwrap_err();

        assert_eq!(
            err,
            DependencyError::ConflictingName {
                name: "one".to_owned()
            }
        );
    }

    #[test]
    fn depdendency_graph_resolve_err_conflict_blob() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = Store::<LocalBackend>::init(store_path).unwrap();

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
            .collect::<Result<Vec<bool>, StoreError>>()
            .unwrap();

        let err = graph
            .resolve(reqs, &store)
            .map(|iter| iter.count())
            .unwrap_err();

        assert_matches!(err, DependencyError::ConflictingBlob { blob: _ });
    }
}
