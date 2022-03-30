pub fn replace_by<T, P: FnOnce() -> T>(dest: &mut T, predicate: P) -> T {
    std::mem::replace(dest, predicate())
}
