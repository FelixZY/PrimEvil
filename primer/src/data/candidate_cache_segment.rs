use crate::data::PrimeCandidate;

/// An iterator over offloaded candidates.
///
/// Contract: This iterator should always be passed in a sorted, ascending, order
pub type CandidateIterator = dyn DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)>;

pub trait CandidateCacheSegment {
    fn has_candidate(&self, candidate: PrimeCandidate) -> bool;

    /// Inserts the given item into this segment.
    ///
    /// Returns `Some` list of elements to offload from the current segment, or `None` if no such
    /// offload is necessary.
    fn add(&mut self, item: (PrimeCandidate, PrimeCandidate)) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)>;

    /// Prepends the given range of items from an offloading cache segment.
    ///
    /// Returns `None` if all elements were consumed. Otherwise, returns `Some` list of elements to
    /// offload from the current segment.
    fn accept_offload(&mut self, offload: &mut CandidateIterator)
        -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)>;

    /// Appends the given range of items from an onloading cache segment.
    ///
    /// Returns `None` if all elements were consumed. Otherwise, returns `Some` list of elements to
    /// onload from the current segment.
    fn accept_onload(&mut self, onload: &mut CandidateIterator) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)>;

    /// Removes and returns the front element from this segment.
    fn pop_front(&mut self) -> Option<(PrimeCandidate, PrimeCandidate)>;

    /// Removes and returns the front elements from this segment.
    fn pull_front(
        &mut self,
        count: usize,
    ) -> impl DoubleEndedIterator<Item = (PrimeCandidate, PrimeCandidate)>;
}
