use serde::{Deserialize, Serialize};
use std::{array::TryFromSliceError, fmt, ops::Deref, str::FromStr};

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

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct PackageId(RawId);

impl PackageId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

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
    type Err = TryFromSliceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = <[u8; 32]>::try_from(s.as_bytes())?;
        Ok(PackageId(RawId(hash)))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, PartialOrd, Ord)]
pub struct ObjectId(RawId);

impl ObjectId {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0 .0
    }

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
    type Err = TryFromSliceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hash = <[u8; 32]>::try_from(s.as_bytes())?;
        Ok(ObjectId(RawId(hash)))
    }
}
