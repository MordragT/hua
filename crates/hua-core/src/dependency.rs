use std::{
    collections::{BTreeSet, HashSet},
    fmt,
    fmt::Debug,
    hash::Hash,
};

use semver::VersionReq;
use serde::{Deserialize, Serialize};

use crate::Component;

// pub trait Requirement: Debug + Send {
//     /// The name of the dependency
//     fn name(&self) -> &String;
//     /// The version required
//     fn version_req(&self) -> &VersionReq;
//     fn requires(&self) -> &HashSet<Requirement>;
//     fn provides(&self) -> &HashSet<Component>;
// }

pub trait Conflicts {
    fn conflicts<'a>(&'a self) -> &dyn Iterator<Item = Conflict<'a>>;
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Conflict<'a> {
    Name(&'a String),
    Component(&'a Component),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub struct Requirement {
    name: String,
    version_req: VersionReq,
    components: BTreeSet<Component>,
    // requires: HashSet<Requirement>,
    // provides: HashSet<Component>,
}

impl Requirement {
    pub fn new(
        name: String,
        version_req: VersionReq,
        // requires: HashSet<Requirement>,
        components: BTreeSet<Component>,
    ) -> Self {
        Self {
            name,
            version_req,
            components,
            // requires,
            // provides,
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }
    pub fn version_req(&self) -> &VersionReq {
        &self.version_req
    }
    pub fn components(&self) -> &BTreeSet<Component> {
        &self.components
    }

    // pub fn requires(&self) -> &HashSet<Requirement> {
    //     &self.requires
    // }
    // pub fn provides(&self) -> &HashSet<Component> {
    //     &self.provides
    // }
}

impl Conflicts for Requirement {
    fn conflicts<'a>(&'a self) -> &dyn Iterator<Item = Conflict<'a>> {
        &self
            .components
            .iter()
            .map(|c| Conflict::Component(c))
            .chain(std::iter::once(Conflict::Name(&self.name)))
    }
    // fn conflicts<'a, I>(&'a self) -> I
    // where
    //     I: Iterator<Item = Conflict<'a>>,
    // {

    // }
}

impl fmt::Display for Requirement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&format!(
            "Dependency {}:\nversion_req: {}\ncomponents: {:#?}\n",
            self.name, self.version_req, self.components
        ))
    }
}

// impl Hash for Requirement {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         state.write(self.name.as_bytes());
//         self.version_req.hash(state);

//         for component in &self.components {
//             component.hash(state);
//         }
//         // for dependency in &self.requires {
//         //     dependency.hash(state);
//         // }
//     }
// }

// impl Eq for dyn Requirement {}

// impl PartialEq for dyn Requirement {
//     fn eq(&self, other: &Self) -> bool {
//         self.name() == other.name()
//             && self.version_req() == other.version_req()
//             && self.requires() == other.requires()
//             && self.provides() == other.provides()
//     }
// }

// impl Clone for Requirement {
//     fn clone(&self) -> Self {
//         Box::new(InternalRequirement {
//             name: self.name().to_owned(),
//             version_req: self.version_req().to_owned(),
//             requires: self.requires().clone(),
//             provides: self.provides().clone(),
//         })
//     }
// }

// impl Serialize for dyn Requirement {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let mut state = serializer.serialize_struct("Dependency", 4)?;

//         state.serialize_field("name", self.name())?;
//         state.serialize_field("version_req", self.version_req())?;
//         state.serialize_field("requires", self.requires())?;
//         state.serialize_field("provides", self.provides())?;

//         state.end()
//     }
// }

// impl<'de> Deserialize<'de> for Requirement {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         enum Field {
//             Name,
//             VersionReq,
//             Requires,
//             Provides,
//         }

//         impl<'de> Deserialize<'de> for Field {
//             fn deserialize<D>(deserializer: D) -> Result<Field, D::Error>
//             where
//                 D: Deserializer<'de>,
//             {
//                 struct FieldVisitor;

//                 impl<'de> Visitor<'de> for FieldVisitor {
//                     type Value = Field;

//                     fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                         formatter.write_str("name, version, requires or provides")
//                     }

//                     fn visit_str<E>(self, value: &str) -> Result<Field, E>
//                     where
//                         E: de::Error,
//                     {
//                         match value {
//                             "name" => Ok(Field::Name),
//                             "version_req" => Ok(Field::VersionReq),
//                             "requires" => Ok(Field::Requires),
//                             "provides" => Ok(Field::Provides),
//                             _ => Err(de::Error::unknown_field(value, FIELDS)),
//                         }
//                     }
//                 }

//                 deserializer.deserialize_identifier(FieldVisitor)
//             }
//         }

//         struct DependencyVisitor;

//         impl<'de> Visitor<'de> for DependencyVisitor {
//             type Value = Requirement;

//             fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//                 formatter.write_str("struct Dependency")
//             }

//             fn visit_seq<V>(self, mut seq: V) -> Result<Self::Value, V::Error>
//             where
//                 V: SeqAccess<'de>,
//             {
//                 let name = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(0, &self))?;
//                 let version_req = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                 let requires = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(2, &self))?;
//                 let provides = seq
//                     .next_element()?
//                     .ok_or_else(|| de::Error::invalid_length(1, &self))?;
//                 Ok(Box::new(InternalRequirement {
//                     name,
//                     version_req,
//                     requires,
//                     provides,
//                 }))
//             }

//             fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
//             where
//                 V: MapAccess<'de>,
//             {
//                 let mut name = None;
//                 let mut version_req = None;
//                 let mut requires = None;
//                 let mut provides = None;

//                 while let Some(key) = map.next_key()? {
//                     match key {
//                         Field::Name => {
//                             if name.is_some() {
//                                 return Err(de::Error::duplicate_field("name"));
//                             }
//                             name = Some(map.next_value()?);
//                         }
//                         Field::VersionReq => {
//                             if version_req.is_some() {
//                                 return Err(de::Error::duplicate_field("version_req"));
//                             }
//                             version_req = Some(map.next_value()?);
//                         }
//                         Field::Requires => {
//                             if requires.is_some() {
//                                 return Err(de::Error::duplicate_field("requires"));
//                             }
//                             requires = Some(map.next_value()?);
//                         }
//                         Field::Provides => {
//                             if provides.is_some() {
//                                 return Err(de::Error::duplicate_field("provides"));
//                             }
//                             provides = Some(map.next_value()?);
//                         }
//                     }
//                 }
//                 let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
//                 let version_req =
//                     version_req.ok_or_else(|| de::Error::missing_field("version_req"))?;
//                 let requires = requires.ok_or_else(|| de::Error::missing_field("requires"))?;
//                 let provides = provides.ok_or_else(|| de::Error::missing_field("provides"))?;

//                 Ok(Box::new(InternalRequirement {
//                     name,
//                     version_req,
//                     requires,
//                     provides,
//                 }))
//             }
//         }

//         const FIELDS: &'static [&'static str] = &["name", "version_req", "requires", "provides"];
//         deserializer.deserialize_struct("Dependency", FIELDS, DependencyVisitor)
//     }
// }
