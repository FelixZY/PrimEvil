use rusqlite::Error;

pub trait I64PrioQueueStorage {
    /// Inserts the given items into storage.
    fn insert(&mut self, items: &Vec<(i64, i64)>) -> Result<(), Error>;

    /// Retrieves up to [count] items from storage.
    ///
    /// The returned vector is sorted in ascending order.
    ///
    /// The returned values are removed from storage.
    fn retrieve(&mut self, count: usize) -> Result<Vec<(i64, i64)>, Error>;

    /// Returns the number of items in storage
    fn len(&self) -> usize;

    /// Returns true if there are no items in storage.
    #[allow(dead_code)]
    fn is_empty(&self) -> bool;

    /// Returns true if there are items in storage.
    #[allow(dead_code)]
    fn is_not_empty(&self) -> bool;
}
