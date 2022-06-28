use super::{step::Step, Conflict, DependencyError, DependencyResult, Requirement};
use crate::store::{backend::ReadBackend, id::PackageId, object::Blob, Store};
use daggy::{Dag, NodeIndex};
use std::collections::{HashMap, HashSet};

/// A directed acyclic graph for dependency resolution.
///
/// # Example
///
/// ```
/// use std::collections::BTreeSet;
/// use semver::VersionReq;
/// use hua_core::dependency::{DependencyGraph, Requirement};
/// use hua_core::store::MemoryStore;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let store = MemoryStore::init()?;
/// let mut graph = DependencyGraph::new();
///
/// let first_req = Requirement::new("first".to_owned(), VersionReq::parse(">0.1.0")?, BTreeSet::new());
/// let second_req = Requirement::new("second".to_owned(), VersionReq::parse(">1.1.0")?, BTreeSet::new());
/// let requirements = [&first_req, &second_req];
///
/// graph.resolve(requirements, &store)?;
/// assert!(!graph.is_resolved());
/// # Ok(())
///# }
/// ```
///
#[derive(Debug, Clone, Default)]
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
    /// Creates a new [DependencyGraph].
    ///
    /// # Example
    ///
    /// ```
    /// use hua_core::dependency::DependencyGraph;
    ///
    /// # fn main() {
    /// let graph = DependencyGraph::new();
    /// # }
    pub fn new() -> Self {
        Self::default()
    }

    /// Resolves multiple [Requirement] with the given [Store].
    ///
    /// # Example
    ///
    /// ```
    /// use std::collections::BTreeSet;
    /// use semver::VersionReq;
    /// use hua_core::dependency::{DependencyGraph, Requirement};
    /// use hua_core::store::MemoryStore;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let store = MemoryStore::init()?;
    /// let mut graph = DependencyGraph::new();
    ///
    /// let first_req = Requirement::new("first".to_owned(), VersionReq::parse(">0.1.0")?, BTreeSet::new());
    /// let second_req = Requirement::new("second".to_owned(), VersionReq::parse(">1.1.0")?, BTreeSet::new());
    /// let requirements = [&first_req, &second_req];
    ///
    /// graph.resolve(requirements, &store)?;
    /// assert!(!graph.is_resolved());
    /// # Ok(())
    ///# }
    /// ```
    pub fn resolve<S, B: ReadBackend>(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store<S, B>,
    ) -> DependencyResult<()> {
        let mut choices = HashMap::new();

        let _nodes = self.resolve_multiple(requirements, store, &mut choices)?;
        self.resolve_choices(choices, store)?;

        // TODO really all choices resolved ?

        Ok(())
    }

    /// Returns all [Requirement] that are still unresolved.
    pub fn unresolved_requirements(&self) -> impl Iterator<Item = &Requirement> {
        self.relations
            .graph()
            .node_weights()
            .filter_map(Step::as_unresolved)
    }

    /// Returns all [PackageId] that are resolved.
    pub fn resolved_packages(&self) -> impl Iterator<Item = PackageId> + '_ {
        self.relations
            .graph()
            .node_weights()
            .filter_map(Step::as_resolved)
    }

    /// Checks if the [DependencyGraph] is resolved.
    pub fn is_resolved(&self) -> bool {
        self.relations.graph().node_weights().all(Step::is_resolved)
    }

    /// Returns all [Step] in the graph.
    pub fn nodes(&self) -> impl Iterator<Item = &Step> {
        self.relations.graph().node_weights()
    }

    fn resolve_multiple<S, B: ReadBackend>(
        &mut self,
        requirements: impl IntoIterator<Item = &'a Requirement>,
        store: &'a Store<S, B>,
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
                        if store.is_matching(id, req) {
                            Some((*id, *index))
                        } else {
                            None
                        }
                    })
                    .collect::<HashMap<PackageId, NodeIndex<usize>>>();

                match inserted_options.len() {
                    0 => self.resolve_single(req, store, choices)?,
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

    fn resolve_single<S, B: ReadBackend>(
        &mut self,
        req: &'a Requirement,
        store: &'a Store<S, B>,
        choices: &mut HashMap<&'a Requirement, NodeIndex<usize>>,
    ) -> DependencyResult<NodeIndex<usize>> {
        let options = store
            .matches(req)
            .map(|(id, drv, blobs)| (id, (drv, blobs)))
            .collect::<HashMap<&PackageId, (_, _)>>();

        let node = match options.len() {
            0 => self.relations.add_node(Step::Unresolved(req)),
            1 => {
                let (id, (package, blobs)) =
                    unsafe { options.into_iter().next().unwrap_unchecked() };
                if let Some(conflict) = self.conflicts(&package.name, blobs) {
                    return Err(conflict)?;
                }

                let node = self.relations.add_node(Step::Resolved(*id));
                self.visited.insert(req, node);
                self.inserted.insert(*id, node);

                let children = self.resolve_multiple(&package.requires, store, choices)?;
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

    fn resolve_choices<S, B: ReadBackend>(
        &mut self,
        choices: HashMap<&'a Requirement, NodeIndex<usize>>,
        store: &'a Store<S, B>,
    ) -> DependencyResult<()> {
        let mut future_choices = HashMap::new();
        for (req, node) in choices {
            let step = unsafe { self.relations.node_weight_mut(node).unwrap_unchecked() };

            let options = unsafe { step.as_choice_unchecked() }.clone();
            let mut result = None;

            for id in options.iter() {
                let drv = store.packages().get(id).unwrap();
                let blobs = unsafe { store.get_blobs_of_package(id).unwrap_unchecked() };
                if conflicts(&mut self.names, &mut self.objects, &drv.name, blobs).is_none() {
                    *step = Step::Resolved(*id);
                    result = Some(drv);
                    break;
                }
            }

            match result {
                None => *step = Step::Unresolved(req),
                Some(drv) => {
                    let children =
                        self.resolve_multiple(&drv.requires, store, &mut future_choices)?;
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

    fn conflicts(
        &mut self,
        name: &'a String,
        blobs: impl Iterator<Item = &'a Blob>,
    ) -> Option<Conflict> {
        conflicts(&mut self.names, &mut self.objects, name, blobs)
    }
}

fn conflicts<'a>(
    names: &mut HashSet<&'a String>,
    objects: &mut HashSet<&'a Blob>,
    name: &'a String,
    blobs: impl Iterator<Item = &'a Blob>,
) -> Option<Conflict<'a>> {
    if !names.insert(name) {
        return Some(Conflict::Name(name));
    }

    for blob in blobs {
        if !objects.insert(blob) {
            return Some(Conflict::Blob(blob));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{LocalStore, StoreError};
    use crate::support::*;
    use std::assert_matches::assert_matches;
    use std::path::PathBuf;
    use temp_dir::TempDir;

    #[test]
    fn dependency_graph_resolve_ok_true() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = LocalStore::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let one_req = req("one", ">0.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [one_req.clone()]);
        let two_req = req("two", ">0.0.0");

        let three_path = temp_dir.child("three");
        let three = pkg("three", &three_path);
        let three_req = req("three", ">0.0.0");

        let four_path = temp_dir.child("four");
        let four = pkg_req("four", &four_path, [one_req.clone(), two_req, three_req]);
        let req_four = req("four", ">0.0.0");

        let reqs = vec![&req_four, &one_req];

        let pkgs = vec![one, two, three, four];
        store
            .extend(pkgs)
            .collect::<Result<Vec<PathBuf>, StoreError>>()
            .unwrap();

        graph.resolve(reqs, &store).unwrap();

        assert!(graph.is_resolved());
        assert_eq!(4, graph.resolved_packages().count());
    }

    #[test]
    fn dependency_graph_resolve_ok_not_resolved() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = LocalStore::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let one_req = req("one", ">0.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("other", ">1.0")]);
        let two_req = req("two", ">0.0.0");

        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![one, two];
        store
            .extend(pkgs)
            .collect::<Result<Vec<PathBuf>, StoreError>>()
            .unwrap();

        graph.resolve(reqs, &store).unwrap();

        assert!(!graph.is_resolved());
    }

    #[test]
    fn dependency_graph_resolve_err_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = LocalStore::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg_req("one", &one_path, [req("two", ">=1.0")]);
        let one_req = req("one", ">0.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req("one", ">=1.0")]);
        let two_req = req("two", ">0.0.0");

        let reqs = vec![&one_req, &two_req];
        let pkgs = vec![one, two];
        store
            .extend(pkgs)
            .collect::<Result<Vec<PathBuf>, StoreError>>()
            .unwrap();

        let err = graph.resolve(reqs, &store).unwrap_err();

        assert_matches!(err, DependencyError::CycleDetected { error: _ });
    }

    #[test]
    fn depdendency_graph_resolve_err_conflict_name() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = LocalStore::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let req_one = req("one", ">=1.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_req("two", &two_path, [req_one]);
        let two_req = req("two", ">0.0.0");

        let three_path = temp_dir.child("three");
        let three = pkg_prov("one", &three_path, "other");
        let three_req = req_comp("one", ">=1.0.0", [Blob::new("lib/other.so".into())]);

        let reqs = vec![&two_req, &three_req];

        let pkgs = vec![one, two, three];
        store
            .extend(pkgs)
            .collect::<Result<Vec<PathBuf>, StoreError>>()
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
    fn depdendency_graph_resolve_err_conflict_blob() {
        let temp_dir = TempDir::new().unwrap();
        let store_path = temp_dir.child("store");

        let mut graph = DependencyGraph::new();
        let mut store = LocalStore::init(store_path).unwrap();

        let one_path = temp_dir.child("one");
        let one = pkg("one", &one_path);
        let one_req = req("one", ">0.0.0");

        let two_path = temp_dir.child("two");
        let two = pkg_prov("two", &two_path, "one");
        let two_req = req_comp("two", ">0.0.0", [Blob::new("lib/one.so".into())]);

        let reqs = vec![&one_req, &two_req];

        let pkgs = vec![one, two];
        store
            .extend(pkgs)
            .collect::<Result<Vec<PathBuf>, StoreError>>()
            .unwrap();

        dbg!(&store);

        let err = graph.resolve(reqs, &store).unwrap_err();

        dbg!(&temp_dir);
        temp_dir.leak();

        assert_matches!(err, DependencyError::ConflictingBlob { blob: _ });
    }
}
