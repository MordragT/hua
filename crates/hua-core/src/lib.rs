mod error;
mod generation;
mod package;
mod store;
mod user;

pub use error::Error;
pub use generation::*;
pub use package::Package;
pub use store::Store;
pub use user::User;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
