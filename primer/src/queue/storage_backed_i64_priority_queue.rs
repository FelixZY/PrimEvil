use crate::queue::dao::I64PrioQueueStorage;
use crate::queue::prio_queue::PriorityQueue;
use std::collections::VecDeque;

/// A priority queue which offloads items exceeding a length limit to storage.
///
/// It is not thread safe.
pub struct StorageBackedI64PriorityQueue {
    queue: VecDeque<(i64, i64)>,
    storage: Box<dyn I64PrioQueueStorage>,
    size: usize,
    offload_limit: usize,
    to_storage_buffer: VecDeque<(i64, i64)>,
}

impl StorageBackedI64PriorityQueue {
    pub fn new(storage: Box<dyn I64PrioQueueStorage>) -> Self {
        Self::with_capacity(storage, 75_000)
    }

    pub fn with_capacity(storage: Box<dyn I64PrioQueueStorage>, capacity: usize) -> Self {
        Self::with_capacity_and_offload_size(storage, capacity, capacity * 1 / 4)
    }

    fn with_capacity_and_offload_size(
        storage: Box<dyn I64PrioQueueStorage>,
        capacity: usize,
        offload_size: usize,
    ) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            size: storage.len(),
            storage,
            offload_limit: offload_size,
            to_storage_buffer: VecDeque::with_capacity(9_000),
        }
    }

    fn fill_queue_from_storage(&mut self) {
        let pull_count = self
            .offload_limit
            .min(self.queue.capacity() - self.queue.len());
        if pull_count > 0 {
            if !self.to_storage_buffer.is_empty() {
                self.storage
                    .insert(&self.to_storage_buffer.drain(..).collect())
                    .expect("insert should succeed");
            }
            self.storage
                .retrieve(pull_count)
                .expect("storage should not be empty")
                .iter()
                .for_each(|&item| {
                    // Assumption: the priority of the back item of the queue is always less than
                    // or equal to the first item of the storage.
                    self.queue.push_back(item);
                });
        }
    }

    /// Offloads items from the back of this queue to storage
    fn offload_back(&mut self) {
        let to_offload = self
            .queue
            .drain((self.queue.len() - self.offload_limit.min(self.queue.len()))..)
            .chain(self.to_storage_buffer.drain(..))
            .collect::<Vec<(i64, i64)>>();
        if !to_offload.is_empty() {
            self.storage
                .insert(&to_offload)
                .expect("insert should succeed");
        }
    }

    fn offload_storage_buffer(&mut self) {
        if !self.to_storage_buffer.is_empty() {
            self.storage
                .insert(&self.to_storage_buffer.drain(..).collect())
                .expect("insert should succeed");
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
        self.size != 0
    }

    fn peek(&mut self) -> Option<&(i64, i64)> {
        if self.is_not_empty() && self.queue.is_empty() {
            self.fill_queue_from_storage();
        }

        self.queue.front()
    }

    fn poll(&mut self) -> Option<(i64, i64)> {
        if self.is_not_empty() && self.queue.is_empty() {
            self.fill_queue_from_storage();
        }

        if let Some(result) = self.queue.pop_front() {
            self.size -= 1;
            return Some(result);
        }
        None
    }

    fn insert(&mut self, item: (i64, i64)) {
        let insertion_point: usize = self
            .queue
            .binary_search_by_key(&item.0, |&(priority, _)| priority)
            .unwrap_or_else(|index| index);

        if insertion_point < self.queue.len() {
            if self.queue.len() == self.queue.capacity() {
                self.offload_back()
            }
            self.queue
                .insert(insertion_point.min(self.queue.len()), item);
        } else if self.queue.len() == self.queue.capacity() {
            self.to_storage_buffer.push_back(item);
            self.offload_back()
        } else if self.size > self.queue.len() {
            // We can't be sure the given item is the smallest of any stored items.
            // Offload it to storage for now and deal with it when the queue is polled for items.
            self.to_storage_buffer.push_back(item);
            if self.to_storage_buffer.len() == self.to_storage_buffer.capacity() {
                self.offload_storage_buffer()
            }
        } else {
            self.queue.push_back(item);
        }

        self.size += 1;
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
        assert_eq!(queue.queue.len(), 0);
    }

    #[test]
    fn insert_one() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        queue.insert((100, 200));

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), 1);
        assert_eq!(queue.queue.len(), 1);
        assert_eq!(queue.storage.len(), 0);
    }

    #[test]
    fn insert_multiple_one_by_one() {
        let inserts = 100usize;

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);
        assert!(inserts < queue.queue.capacity());

        for i in 0..inserts {
            queue.insert((i as i64, 100 + i as i64));
            assert!(!queue.is_empty());
            assert!(queue.is_not_empty());
            assert_eq!(queue.len(), i + 1);
            assert_eq!(queue.queue.len(), i + 1);
            assert_eq!(queue.storage.len(), 0);
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
        assert_eq!(queue.queue.len(), 2);
        assert_eq!(queue.storage.len(), 0);
    }

    #[test]
    fn insert_all_multiple_one_by_one() {
        let inserts = 100usize;

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);
        assert!(inserts < queue.queue.capacity());

        for i in 0..inserts {
            queue.insert((i as i64, 100 + i as i64));
            assert!(!queue.is_empty());
            assert!(queue.is_not_empty());
            assert_eq!(queue.len(), i + 1);
            assert_eq!(queue.queue.len(), i + 1);
            assert_eq!(queue.storage.len(), 0);
        }
    }

    #[test]
    fn insert_multiple_at_once() {
        let inserts = 100usize;

        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);
        assert!(inserts < queue.queue.capacity());

        (0..inserts)
            .map(|i| (i as i64, 100 + i as i64))
            .for_each(|item| queue.insert(item));

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), inserts);
        assert_eq!(queue.queue.len(), inserts);
        assert_eq!(queue.storage.len(), 0);
    }

    #[test]
    fn peek() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        vec![(900, 900), (100, 200), (500, 600), (300, 400), (700, 800)]
            .iter()
            .for_each(|&item| queue.insert(item));

        assert_eq!(queue.len(), 5);
        assert_eq!(queue.peek(), Some(&(100, 200)));
        assert_eq!(queue.len(), 5, "Peek must not remove items");
        assert_eq!(queue.queue.len(), 5);
        assert_eq!(queue.storage.len(), 0);
    }

    #[test]
    fn poll() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);

        vec![(900, 900), (100, 200), (500, 600), (300, 400), (700, 800)]
            .iter()
            .for_each(|&item| queue.insert(item));

        for i in 0..5 {
            let item = queue.poll().expect("queue should not be empty");
            assert_eq!(queue.len(), 4 - i);
            assert_eq!(queue.queue.len(), 4 - i);
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
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);
        let initial_capacity = queue.queue.capacity();
        let inserts = initial_capacity * 2;

        (0..inserts)
            .map(|i| (i as i64, 100 + i as i64))
            .for_each(|item| queue.insert(item));

        assert!(!queue.is_empty());
        assert!(queue.is_not_empty());
        assert_eq!(queue.len(), inserts);
        assert!(!queue.queue.is_empty());
        assert!(queue.storage.is_not_empty());
        assert_eq!(queue.queue.capacity(), initial_capacity);
    }

    #[test]
    fn polls_in_correct_order() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::new(dao);
        let initial_capacity = queue.queue.capacity();
        let inserts = initial_capacity * 2;

        // Insert in reverse order to ensure sorting takes place on retrieval
        (0..inserts)
            .rev()
            .map(|i| (i as i64, 100 + i as i64))
            .for_each(|item| queue.insert(item));

        assert_eq!(queue.queue.capacity(), initial_capacity);

        for i in 0..inserts {
            let item = queue.poll().expect("queue should not be empty");
            assert_eq!(item.0, i as i64);
            assert_eq!(queue.len(), inserts - i - 1);
        }

        assert!(queue.is_empty());
        assert!(queue.queue.is_empty());
        assert!(queue.storage.is_empty());
        assert_eq!(queue.queue.capacity(), initial_capacity);
    }

    #[test]
    fn push_higher_than_storage_lowest() {
        let dao = get_dao();
        let mut queue = StorageBackedI64PriorityQueue::with_capacity_and_offload_size(dao, 4, 1);
        let initial_capacity = queue.queue.capacity();

        // Fill queue with low prio values
        (0..initial_capacity - 1)
            .map(|_| (1, 0))
            .for_each(|item| queue.insert(item));
        assert!(queue.to_storage_buffer.is_empty());
        assert!(queue.storage.is_empty());

        // Insert a high prio value at the end
        queue.insert((3, 0));
        assert_eq!(queue.queue.len(), initial_capacity);
        assert!(queue.to_storage_buffer.is_empty());
        assert!(queue.storage.is_empty());

        // Bump 3 to storage (since 2 < 3)
        queue.insert((2, 0));
        assert_eq!(queue.queue.len(), initial_capacity);
        assert!(queue.to_storage_buffer.is_empty());
        assert_eq!(queue.storage.len(), 1);

        // Open up a spot in the queue
        queue.poll().expect("queue should not be empty");
        assert_eq!(queue.queue.len(), initial_capacity - 1);
        assert!(queue.to_storage_buffer.is_empty());
        assert_eq!(queue.storage.len(), 1);

        // Actual test: make sure 4 is bumped to storage since it is larger than 3
        // (which was bumped to storage previously)
        queue.insert((4, 0));
        assert_eq!(queue.queue.len(), initial_capacity - 1);
        assert_eq!(queue.to_storage_buffer.len(), 1);
        assert_eq!(queue.storage.len(), 1);

        // Actual test: make sure items are retrieved in the correct order
        while queue.is_not_empty() {
            let (priority, _) = queue.poll().expect("queue should not be empty");
            match queue.size {
                2 => assert_eq!(priority, 2),
                1 => assert_eq!(priority, 3),
                0 => assert_eq!(priority, 4),
                _ => assert_eq!(priority, 1),
            }
        }
    }

    fn get_dao() -> Box<SqlitePrioQueueDao> {
        Box::new(SqlitePrioQueueDao::new(":memory:"))
    }
}
