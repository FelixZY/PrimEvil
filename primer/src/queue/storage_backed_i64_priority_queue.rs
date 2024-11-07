use std::collections::Bound::Unbounded;
use std::ops::Bound;
use crate::queue::dao::I64PrioQueueStorage;
use crate::queue::prio_queue::PriorityQueue;
use skiplist::OrderedSkipList;

const DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD: usize = 5_000;
const DEFAULT_LOAD_FROM_STORAGE_THRESHOLD: usize = 1_000;

/// A priority queue which offloads items exceeding a length limit to storage.
///
/// It is not thread safe.
pub struct StorageBackedI64PriorityQueue {
    map: OrderedSkipList<(i64, i64)>,
    storage: Box<dyn I64PrioQueueStorage>,
    size: usize,
    offload_to_storage_threshold: usize,
    load_from_storage_threshold: usize,
}

impl StorageBackedI64PriorityQueue {
    pub fn new(storage: Box<dyn I64PrioQueueStorage>) -> Self {
        Self::new_with_thresholds(
            storage,
            DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD,
            DEFAULT_LOAD_FROM_STORAGE_THRESHOLD,
        )
    }

    pub fn new_with_thresholds(
        storage: Box<dyn I64PrioQueueStorage>,
        offload_to_storage_threshold: usize,
        load_from_storage_threshold: usize,
    ) -> Self {
        assert!(offload_to_storage_threshold > 0);
        assert!(load_from_storage_threshold > 0);
        assert!(offload_to_storage_threshold > load_from_storage_threshold);
        Self {
            map: unsafe { OrderedSkipList::<(i64, i64)>::with_comp(|(a, _), (b, _)| a.cmp(b)) },
            size: storage.len(),
            storage,
            offload_to_storage_threshold,
            load_from_storage_threshold,
        }
    }

    fn sync_with_storage(&mut self) {
        // Hit storage for length once only
        let storage_len = self.storage.len();

        if storage_len == 0 {
            self.size = self.map.len();
            return;
        }

        self.size = self.map.len() + storage_len;

        if self.map.is_empty() {
            self.storage
                .retrieve(self.offload_to_storage_threshold)
                .expect("storage should not be empty")
                .iter()
                .for_each(|&(priority, value)| {
                    self.map.insert((priority, value));
                });
            return;
        }

        let storage_lowest_prio = self
            .storage
            .lowest_priority()
            .expect("storage should not be empty");
        if self
            .map
            .back()
            .map(|(priority, _)| *priority <= storage_lowest_prio)
            .expect("map should not be empty")
        {
            return;
        }
        let diverging_index =
            self
                .map
                .range(Bound::Excluded(storage_lowest_prio), Unbounded)
                .map(|(priority, _)| *priority <= storage_lowest_prio)
                .expect("map should not be empty");
        
        let mut to_storage =
            Vec::with_capacity(self.map.len() - self.offload_to_storage_threshold);
        while self.map.len() > self.offload_to_storage_threshold {
            to_storage.push(self.map.pop_back().expect("map should contain item"));
        }
        self.storage
            .insert(&to_storage)
            .expect("insert should succeed")      
        
    }

    fn load_from_storage(&mut self, min_priority: usize) {
        if self.size > self.map.len() {
            self.storage
                .retrieve(self.offload_to_storage_threshold - self.map.len())
                .expect("storage should not be empty")
                .iter()
                .for_each(|&(priority, value)| {
                    self.map.insert((priority, value));
                })
        }
    }

    fn offload_to_storage_if_needed(&mut self) {
        if self.map.len() > self.offload_to_storage_threshold {
            let mut to_storage =
                Vec::with_capacity(self.map.len() - self.offload_to_storage_threshold);
            while self.map.len() > self.offload_to_storage_threshold {
                to_storage.push(self.map.pop_back().expect("map should contain item"));
            }
            self.storage
                .insert(&to_storage)
                .expect("insert should succeed")
        }
    }
}

impl PriorityQueue for StorageBackedI64PriorityQueue {
    fn len(&self) -> usize {
        self.size
    }

    fn is_empty(&self) -> bool {
        self.size == 0
    }

    fn is_not_empty(&self) -> bool {
        !self.is_empty()
    }

    fn peek(&mut self) -> Option<&(i64, i64)> {
        if self.map.len() < self.load_from_storage_threshold {
            self.sync_with_storage()
        }

        self.map.front()
    }

    fn poll(&mut self) -> Option<(i64, i64)> {
        if self.map.len() < self.load_from_storage_threshold {
            self.sync_with_storage()
        }

        if let Some(result) = self.map.pop_front() {
            self.size -= 1;
            return Some(result);
        }
        None
    }

    fn insert(&mut self, item: (i64, i64)) {
        if self.is_not_empty() && self.map.is_empty()
            || !self.map.is_empty()
                && self
                    .map
                    .back()
                    .map(|(priority, _)| *priority > item.0)
                    .expect("map should contain item")
        {
            self.sync_with_storage()
        }

        self.size += 1;
        self.map.insert(item);

        self.offload_to_storage_if_needed();
    }

    fn insert_all(&mut self, items: &Vec<(i64, i64)>) {
        if self.is_not_empty() && self.map.is_empty()
            || !self.map.is_empty()
                && self
                    .map
                    .back()
                    .map(|(priority, _)| *priority)
                    .map(|priority| {
                        items
                            .iter()
                            .any(|(new_priority, _)| *new_priority < priority)
                    })
                    .expect("map should contain item")
        {
            self.sync_with_storage()
        }

        items.iter().for_each(|&item| {
            self.size += 1;
            self.map.insert(item);
        });

        self.offload_to_storage_if_needed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::SqlitePrioQueueDao;

    #[test]
    fn queue_starts_empty() {
        let dao = get_dao();
        let queue = StorageBackedI64PriorityQueue::new(dao);

        assert!(queue.is_empty());
        assert!(!queue.is_not_empty());
        assert_eq!(queue.len(), 0);
        assert_eq!(queue.map.len(), 0);
    }

    #[test]
    fn insert_one() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert((100, 200));

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.map.len(), 1);
    }

    #[test]
    fn insert_multiple_one_by_one() {
        let inserts = 100usize;
        assert!(inserts < DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        for i in 0..inserts {
            queue.insert((i as i64, 100 + i as i64));
            assert!(!queue.is_empty());
            assert!(queue.is_not_empty());
            assert_eq!(queue.len(), i + 1);
            assert_eq!(queue.map.len(), i + 1);
        }
    }

    #[test]
    fn insert_duplicate_priority() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert((100, 200));
        queue.insert((100, 400));

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), 2);
        assert_eq!(queue.map.len(), 2);
    }

    #[test]
    fn insert_all_multiple_one_by_one() {
        let inserts = 100usize;
        assert!(inserts < DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        for i in 0..inserts {
            queue.insert_all(&vec![(i as i64, 100 + i as i64)]);
            assert!(!queue.is_empty());
            assert!(queue.is_not_empty());
            assert_eq!(queue.len(), i + 1);
            assert_eq!(queue.map.len(), i + 1);
        }
    }

    #[test]
    fn insert_multiple_at_once() {
        let inserts = 100usize;
        assert!(inserts < DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert_all(&(0..inserts).map(|i| (i as i64, 100 + i as i64)).collect());

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), inserts);
        assert_eq!(queue.map.len(), inserts);
    }

    #[test]
    fn peek() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert_all(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ]);

        assert_eq!(queue.len(), 5);
        assert_eq!(queue.peek(), Some(&(100, 200)));
        assert_eq!(queue.len(), 5, "Peek must not remove items");
        assert_eq!(queue.map.len(), 5);
    }

    #[test]
    fn poll() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert_all(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ]);

        for i in 0..5 {
            let item = queue.poll().expect("queue should not be empty");
            assert_eq!(queue.len(), 4 - i);
            assert_eq!(queue.map.len(), 4 - i);
            assert_eq!(
                item,
                *vec![(100, 200), (300, 400), (500, 600), (700, 800), (900, 900)]
                    .get(i)
                    .unwrap()
            );

            if i < 4 {
                assert!(!queue.is_empty());
                assert!(queue.is_not_empty());
            } else {
                assert!(queue.is_empty());
                assert!(!queue.is_not_empty());
            }
        }

        assert!(queue.poll().is_none());
    }

    #[test]
    fn is_empty_false_when_items_exist_in_storage() {
        let mut dao = get_dao();
        dao.insert(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ])
        .expect("dao should accept inserts");

        let queue = StorageBackedI64PriorityQueue::new(dao);
        assert!(!queue.is_empty());
    }

    #[test]
    fn is_not_empty_true_when_items_exist_in_storage() {
        let mut dao = get_dao();
        dao.insert(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ])
        .expect("dao should accept inserts");

        let queue = StorageBackedI64PriorityQueue::new(dao);
        assert!(queue.is_not_empty());
    }

    #[test]
    fn correct_len_when_items_exist_in_storage() {
        let mut dao = get_dao();
        dao.insert(&vec![
            (900, 900),
            (100, 200),
            (500, 600),
            (300, 400),
            (700, 800),
        ])
        .expect("dao should accept inserts");

        let queue = StorageBackedI64PriorityQueue::new(dao);
        assert_eq!(queue.len(), 5);
    }

    #[test]
    fn offloads_to_storage() {
        let inserts = DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD * 2;
        assert!(inserts > DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert_all(&(0..inserts).map(|i| (i as i64, 100 + i as i64)).collect());

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), inserts);
        assert_eq!(queue.map.len(), DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);
    }

    #[test]
    fn polls_in_correct_order() {
        let inserts = DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD * 2;
        assert!(inserts > DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        // Insert in reverse order to ensure sorting takes place on retrieval
        queue.insert_all(
            &(0..inserts)
                .rev()
                .map(|i| (i as i64, 100 + i as i64))
                .collect(),
        );

        for i in 0..inserts {
            let item = queue.poll().expect("queue should not be empty");
            assert_eq!(item.0, i as i64);
            assert_eq!(queue.len(), inserts - i - 1);
            assert!(queue.map.len() <= DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);
            assert!(queue.storage.len() <= DEFAULT_OFFLOAD_TO_STORAGE_THRESHOLD);
        }

        assert!(queue.is_empty());
        assert!(queue.map.is_empty());
        assert!(queue.storage.is_empty());
    }

    #[test]
    fn asdf() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new_with_thresholds(dao, 2, 1);

        queue.insert_all(&vec![(1, 0), (3, 0)]);

        queue.insert((2, 0));
        queue.poll().expect("queue should not be empty");
        queue.insert((4, 0));

        assert_eq!(
            queue.map.iter().map(|&x| x).collect::<Vec<(i64, i64)>>(),
            vec![(2, 0), (3, 0)]
        );
    }

    fn get_dao() -> Box<SqlitePrioQueueDao> {
        Box::new(SqlitePrioQueueDao::new(":memory:"))
    }
}
