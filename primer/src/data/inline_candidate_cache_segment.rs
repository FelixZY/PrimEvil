use crate::data::binary_searchable::BinarySearchable;
use crate::data::candidate_cache_segment::{CandidateCacheSegment, CandidateIterator};
use crate::data::PrimeCandidate;
use arraydeque::ArrayDeque;
use log::debug;
use std::iter::{empty, once};

/// "Inline" in the sense that data is stored directly on the stack rather than behind a reference
/// on the heap.
#[derive(Debug)]
struct InlineCandidateCacheSegment<const N: usize> {
    cache: ArrayDeque<(PrimeCandidate, PrimeCandidate), N>,
    offload_count: usize,
}

impl<const N: usize> InlineCandidateCacheSegment<N> {
    pub fn new() -> Self {
        assert!(N > 0);

        let instance = Self {
            cache: ArrayDeque::new(),
            offload_count: (N as f32 / 2f32).ceil() as usize,
        };

        instance
    }
}

impl<const N: usize> CandidateCacheSegment for InlineCandidateCacheSegment<N> {
    fn has_candidate(&self, candidate: PrimeCandidate) -> bool {
        self.binary_search(&candidate, 0, self.len().checked_sub(1).unwrap_or(0))
            .is_ok()
    }

    fn add(
        &mut self,
        item: (PrimeCandidate, PrimeCandidate),
    ) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)> {
        let insertion_point = self
            .binary_search(&item.0, 0, self.len().checked_sub(1).unwrap_or(0))
            .unwrap_or_else(|i| i);

        if let Err(_) = self.cache.insert(insertion_point, item) {
            let is_insertion_point_to_be_offloaded =
                insertion_point > self.len() - self.offload_count;
            let item_vec = if is_insertion_point_to_be_offloaded {
                vec![item]
            } else {
                vec![]
            };
            let result = self
                .cache
                .drain((self.len() - self.offload_count + item_vec.len())..)
                .chain(item_vec)
                .collect::<Vec<_>>();

            if !is_insertion_point_to_be_offloaded {
                self.cache
                    .insert(insertion_point.min(self.len()), item)
                    .expect("Insert should succeed after drain");
            }

            return result.into_iter();
        }
        vec![].into_iter()
    }

    fn accept_offload(
        &mut self,
        offload: &mut CandidateIterator,
    ) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)> {
        offload
            // It's more efficient to go large -> small as this allows us to call push_front.
            .rev()
            .filter_map(|item| {
                debug_assert!(
                    item.0
                        <= self
                            .cache
                            .front()
                            .map(|&(candidate, _)| candidate)
                            .unwrap_or(PrimeCandidate::MAX)
                );
                if let Err(_) = self.cache.push_front(item) {
                    let result = self
                        .cache
                        .drain((self.len() - self.offload_count)..)
                        .collect::<Vec<_>>();

                    self.cache
                        .push_front(item)
                        .expect("Insert should succeed after drain");

                    return Some(result);
                };
                None
            })
            // The largest values will be kicked out first.
            // This means that while each individual iterator is sorted,
            // the iterators themselves are returned in reverse order.
            .rev()
            .flatten()
    }

    fn accept_onload(
        &mut self,
        onload: &mut CandidateIterator,
    ) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)> {
        onload
            .filter_map(|item| {
                debug_assert!(
                    item.0
                        >= self
                            .cache
                            .back()
                            .map(|&(candidate, _)| candidate)
                            .unwrap_or(0)
                );
                if let Err(_) = self.cache.push_back(item) {
                    let to_onload = self.cache.drain(0..self.offload_count).collect::<Vec<_>>();
                    self.cache
                        .push_back(item)
                        .expect("Insert should succeed after drain");
                    return Some(to_onload);
                }
                None
            })
            .flatten()
    }

    fn pop_front(&mut self) -> Option<(PrimeCandidate, PrimeCandidate)> {
        #[cfg(debug_assertions)]
        {
            if self.len() == 1 {
                debug!("Popping final element from {:?}", self)
            }
        }

        self.cache.pop_front()
    }

    fn pull_front(
        &mut self,
        count: usize,
    ) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)> {
        self.cache.drain(0..count.min(self.len()))
    }
}

impl<const N: usize> BinarySearchable<PrimeCandidate> for InlineCandidateCacheSegment<N> {
    fn len(&self) -> usize {
        self.cache.len()
    }

    fn get(&self, index: usize) -> Option<&PrimeCandidate> {
        self.cache.get(index).map(|(candidate, _)| candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn starts_empty() {
        let segment = InlineCandidateCacheSegment::<5>::new();
        assert_eq!(segment.len(), 0);
    }

    #[test]
    fn insert_1_item_with_capacity_1() {
        insert_n_items_with_capacity_n::<1>()
    }

    #[test]
    fn insert_5_item_with_capacity_5() {
        insert_n_items_with_capacity_n::<5>()
    }

    fn insert_n_items_with_capacity_n<const N: usize>() {
        let mut segment = InlineCandidateCacheSegment::<N>::new();
        let mut added = Vec::with_capacity(N);

        for i in 1..=N {
            let elm = (0, 0);
            added.push(elm);
            assert!(segment.add(elm).collect::<Vec<_>>().is_empty());
            assert_eq!(segment.len(), i);
        }
    }

    #[test]
    fn insert_2_items_with_capacity_1() {
        insert_n_plus_1_items_with_capacity_n::<1, 1>();
    }

    #[test]
    fn insert_6_items_with_capacity_5() {
        insert_n_plus_1_items_with_capacity_n::<5, 3>();
    }

    fn insert_n_plus_1_items_with_capacity_n<
        const N: usize,
        const EXPECTED_OFFLOAD_COUNT: usize,
    >() {
        let mut segment = InlineCandidateCacheSegment::<N>::new();
        assert_eq!(segment.offload_count, EXPECTED_OFFLOAD_COUNT);

        let mut added = Vec::with_capacity(N);
        for i in 1..=N {
            let elm = (0, 0);
            added.push(elm);
            assert!(segment.add(elm).collect::<Vec<_>>().is_empty());
            assert_eq!(segment.len(), i);
        }

        assert_eq!(
            segment.add((0, 0)).collect::<Vec<_>>(),
            std::iter::repeat((0, 0))
                .take(segment.offload_count)
                .collect::<Vec<_>>()
        );
        assert_eq!(segment.len(), N - segment.offload_count + 1);
    }

    #[test]
    fn offloads_largest_when_capacity_is_1() {
        offloads_largest_when_capacity_is::<1, 1>();
    }

    #[test]
    fn offloads_largest_when_capacity_is_5() {
        offloads_largest_when_capacity_is::<5, 3>();
    }

    fn offloads_largest_when_capacity_is<const N: usize, const EXPECTED_OFFLOAD_COUNT: usize>() {
        let mut segment = InlineCandidateCacheSegment::<N>::new();
        assert_eq!(segment.len(), 0);
        assert_eq!(segment.offload_count, EXPECTED_OFFLOAD_COUNT);

        fn fill_segment<const N: usize>(segment: &mut InlineCandidateCacheSegment<N>) {
            for i in (segment.len() as PrimeCandidate + 1)..=(N as PrimeCandidate) {
                let elm = (i, 0);
                assert!(segment.add(elm).collect::<Vec<_>>().is_empty());
            }
            assert_eq!(segment.len(), N);
        }
        fill_segment(&mut segment);

        let mut elm: (PrimeCandidate, PrimeCandidate);
        let mut kickout: Vec<(PrimeCandidate, PrimeCandidate)>;

        // Inserting a higher value causes this higher value to be kicked out
        elm = ((N + 1) as PrimeCandidate, 0);
        kickout = segment.add(elm).collect::<Vec<_>>();
        assert_eq!(kickout.len(), segment.offload_count);
        assert_eq!(
            kickout,
            (((N - segment.offload_count + 2) as PrimeCandidate)..=elm.0)
                .map(|i| (i, 0))
                .collect::<Vec<_>>()
        );
        assert!(
            kickout.windows(2).all(|w| w[0].0 <= w[1].0),
            "Kickout was not sorted! {:?}",
            kickout
        );
        fill_segment(&mut segment);

        // Inserting a lower value causes other higher values to be kicked out
        elm = (0, 0);
        kickout = segment.add(elm).collect::<Vec<_>>();
        assert_eq!(kickout.len(), segment.offload_count);
        assert_eq!(
            kickout,
            (((N - segment.offload_count + 1) as PrimeCandidate)..=(N as PrimeCandidate))
                .map(|i| (i, 0))
                .collect::<Vec<_>>()
        );
        assert!(
            kickout.windows(2).all(|w| w[0].0 <= w[1].0),
            "Kickout was not sorted! {:?}",
            kickout
        );
        fill_segment(&mut segment);
    }
}
