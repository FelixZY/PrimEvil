pub trait PriorityQueue {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn is_not_empty(&self) -> bool;
    fn peek(&mut self) -> Option<&(i64, i64)>;
    fn poll(&mut self) -> Option<(i64, i64)>;
    fn insert(&mut self, item: (i64, i64));
    fn insert_all(&mut self, items: &Vec<(i64, i64)>);
}
