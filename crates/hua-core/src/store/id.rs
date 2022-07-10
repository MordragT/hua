use serde::{Deserialize, Serialize};
use std::{array::TryFromSliceError, fmt, ops::Deref, str::FromStr};

use crate::extra;

use super::location::Source;

/// An raw identifier which can be casted into [PackageId] or [ObjectId].
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct RawId([u8; 32]);

impl From<[u8; 32]> for RawId {
    fn from(hash: [u8; 32]) -> Self {
        RawId(hash)
    }
}

impl From<RawId> for Vec<u8> {
    fn from(id: RawId) -> Self {
        id.0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for RawId {
    type Error = Vec<u8>;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(vec)?;
        Ok(RawId(hash))
    }
}

impl From<PackageId> for RawId {
    fn from(id: PackageId) -> Self {
        id.0
    }
}

impl From<ObjectId> for RawId {
    fn from(id: ObjectId) -> Self {
        id.0
    }
}

/// An identifier for [packages](crate::store::Package).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PackageId(RawId);

impl PackageId {
    /// Converts an [PackageId] to a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

    /// Truncates an [PackageId] into a [u64] number.
    pub fn truncate(&self) -> u64 {
        let mut res: [u8; 8] = Default::default();
        res.copy_from_slice(&self.0 .0[0..8]);
        u64::from_be_bytes(res)
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for PackageId {
    fn from(hash: [u8; 32]) -> Self {
        PackageId(RawId(hash))
    }
}

impl From<PackageId> for Vec<u8> {
    fn from(id: PackageId) -> Self {
        id.0 .0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for PackageId {
    type Error = Vec<u8>;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(vec)?;
        Ok(PackageId(RawId(hash)))
    }
}

impl TryFrom<&[u8]> for PackageId {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(slice)?;
        Ok(PackageId(RawId(hash)))
    }
}

impl From<RawId> for PackageId {
    fn from(id: RawId) -> Self {
        PackageId(id)
    }
}

impl Deref for PackageId {
    type Target = RawId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for PackageId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = extra::str::parse_hex(s);
        vec.try_into()
            .map_err(|_| "Could not convert vec to id.".to_owned())
    }
}

/// An identifier for a [Store](super::Store).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct StoreId(RawId);

impl StoreId {
    /// Converts an [PackageId] to a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

    /// Truncates an [PackageId] into a [u64] number.
    pub fn truncate(&self) -> u64 {
        let mut res: [u8; 8] = Default::default();
        res.copy_from_slice(&self.0 .0[0..8]);
        u64::from_be_bytes(res)
    }
}

impl fmt::Display for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for StoreId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for StoreId {
    fn from(hash: [u8; 32]) -> Self {
        StoreId(RawId(hash))
    }
}

impl From<StoreId> for Vec<u8> {
    fn from(id: StoreId) -> Self {
        id.0 .0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for StoreId {
    type Error = Vec<u8>;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(vec)?;
        Ok(StoreId(RawId(hash)))
    }
}

impl TryFrom<&[u8]> for StoreId {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(slice)?;
        Ok(StoreId(RawId(hash)))
    }
}

impl From<RawId> for StoreId {
    fn from(id: RawId) -> Self {
        StoreId(id)
    }
}

impl Deref for StoreId {
    type Target = RawId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for StoreId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = extra::str::parse_hex(s);
        vec.try_into()
            .map_err(|_| "Could not convert vec to id.".to_owned())
    }
}

/// An identifier for [objects](crate::store::Object).
/// Can be used to retrieve [objects](crate::store::Object)
/// from the [Objects](crate::store::object::Objects) data structure.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ObjectId(RawId);

impl ObjectId {
    /// Converts an [ObjectId] to a slice of bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

    /// Truncates an [ObjectId] into a [u64] number.
    pub fn truncate(&self) -> u64 {
        let mut res: [u8; 8] = Default::default();
        res.copy_from_slice(&self.0 .0[0..8]);
        u64::from_be_bytes(res)
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl fmt::Debug for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in &self.0 .0 {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl From<[u8; 32]> for ObjectId {
    fn from(hash: [u8; 32]) -> Self {
        ObjectId(RawId(hash))
    }
}

impl From<ObjectId> for Vec<u8> {
    fn from(id: ObjectId) -> Self {
        id.0 .0.to_vec()
    }
}

impl TryFrom<Vec<u8>> for ObjectId {
    type Error = Vec<u8>;

    fn try_from(vec: Vec<u8>) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(vec)?;
        Ok(ObjectId(RawId(hash)))
    }
}

impl TryFrom<&[u8]> for ObjectId {
    type Error = TryFromSliceError;

    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        let hash = <[u8; 32]>::try_from(slice)?;
        Ok(ObjectId(RawId(hash)))
    }
}

impl From<RawId> for ObjectId {
    fn from(id: RawId) -> Self {
        ObjectId(id)
    }
}

impl Deref for ObjectId {
    type Target = RawId;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromStr for ObjectId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let vec = extra::str::parse_hex(s);
        vec.try_into()
            .map_err(|_| "Could not convert vec to id.".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::PackageId;

    #[test]
    fn from_str() {
        let id = "cff5b89509a60d44ae34be0e50356287de1e03622bc658e6422b2510cc15ef83";
        let _id = PackageId::from_str(id).unwrap();
    }
}
