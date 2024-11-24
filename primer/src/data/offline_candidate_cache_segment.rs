use crate::data::candidate_cache_segment::CandidateCacheSegment;
use crate::data::PrimeCandidate;
use std::collections::VecDeque;

/// "Offline" in the sense that data is stored behind a pointer, on the heap, rather than
/// immediately on the stack.
struct OfflineCandidateCacheSegment {
    cache: VecDeque<(PrimeCandidate, PrimeCandidate)>,
    min: PrimeCandidate,
    max: PrimeCandidate,
}

impl OfflineCandidateCacheSegment {
    fn new(capacity: usize) -> Self {
        Self {
            cache: VecDeque::with_capacity(capacity),
            min: 0,
            max: PrimeCandidate::MAX,
        }
    }
}

#[cfg(test)]
mod tests {}
