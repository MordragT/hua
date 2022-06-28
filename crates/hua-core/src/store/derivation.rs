use super::id::{DerivationId, PackageId};
use crate::dependency::Requirement;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt,
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Derivation {
    pub requires: HashSet<Requirement>,
}

impl Derivation {
    pub fn new(requires: impl IntoIterator<Item = Requirement>) -> Self {
        Self {
            requires: requires.into_iter().collect(),
        }
    }
}

impl fmt::Display for Derivation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Derivation\n")?;
        for req in &self.requires {
            write!(f, "\tRequirement: {req}\n")?;
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Derivations {
    nodes: HashMap<DerivationId, Derivation>,
    provides: HashMap<DerivationId, HashSet<PackageId>>,
}

impl Derivations {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get(&self, id: &DerivationId) -> Option<&Derivation> {
        self.nodes.get(id)
    }

    pub fn insert(
        &mut self,
        id: DerivationId,
        drv: Derivation,
        provides: impl IntoIterator<Item = PackageId>,
    ) -> Option<Derivation> {
        if let Some(drv) = self.nodes.insert(id, drv) {
            Some(drv)
        } else {
            self.provides.insert(id, provides.into_iter().collect());
            None
        }
    }

    pub fn get_drv_of_pkg(&self, package_id: &PackageId) -> Option<DerivationId> {
        self.provides.iter().find_map(|(drv_id, provides)| {
            if provides.contains(package_id) {
                Some(*drv_id)
            } else {
                None
            }
        })
    }
}
