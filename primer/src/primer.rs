use crate::queue::{PriorityQueue, StorageBackedI64PriorityQueue};
use crate::storage::SqlitePrioQueueDao;

const STEP_SIZE: &[i64] = &[2, 4, 2, 4, 6, 2, 6, 4];
const PREDEFINED_PRIMES: &[i64] = &[2, 3, 5, 7, 11];

pub struct Primer {
    false_candidates: Box<dyn PriorityQueue>,
    last_candidate: i64,
    step_index: usize,
    prime_index: usize,
}

impl Primer {
    pub fn new() -> Self {
        Self::new_with_priority_queue(|| {
            Box::new(StorageBackedI64PriorityQueue::new(Box::new(
                SqlitePrioQueueDao::new(":memory:"),
            )))
        })
    }

    pub fn new_with_priority_queue(
        priority_queue_provider: fn() -> Box<dyn PriorityQueue>,
    ) -> Self {
        Self {
            false_candidates: priority_queue_provider(),
            last_candidate: 0,
            step_index: 0,
            prime_index: 0,
        }
    }

    pub fn crunch<F>(&mut self, chunk_size: usize, mut on_chunk: F) -> ()
    where
        F: FnMut(usize, &Vec<i64>) -> bool,
    {
        let mut chunk = Vec::<i64>::with_capacity(chunk_size);
        while self.prime_index < PREDEFINED_PRIMES.len() {
            let prime = PREDEFINED_PRIMES[self.prime_index];
            self.last_candidate = prime;

            // 2, 3 and 5 are accounted for via STEP_SIZE
            if prime > 5 {
                self.false_candidates.insert((prime * prime, prime));
            }

            chunk.push(prime);
            self.prime_index += 1;
            if chunk.len() == chunk.capacity() {
                if !on_chunk(self.prime_index - chunk.len(), &chunk) {
                    return;
                }
                chunk.clear();
            }
        }

        let mut lowest = self
            .false_candidates
            .peek()
            .map(|(false_candidate, _)| *false_candidate)
            .expect("there will always be false candidates");
        loop {
            let candidate = self.last_candidate + STEP_SIZE[self.step_index];
            let mut candidate_can_be_prime = true;
            self.last_candidate = candidate;
            self.step_index = (self.step_index + 1) % STEP_SIZE.len();

            while candidate >= lowest {
                if candidate == lowest {
                    candidate_can_be_prime = false;
                }

                let (key, prime) = self
                    .false_candidates
                    .poll()
                    .expect("there will always be false candidates");
                // Candidate is always odd.
                // Primes above 2 cannot be even numbers.
                // Multiply existing prime by 2 to optimize for this.
                self.false_candidates.insert((key + prime * 2, prime));

                lowest = self
                    .false_candidates
                    .peek()
                    .map(|(false_candidate, _)| *false_candidate)
                    .expect("there will always be false candidates");
            }

            if !candidate_can_be_prime {
                continue;
            }

            self.false_candidates
                .insert((candidate * candidate, candidate));

            chunk.push(candidate);
            self.prime_index += 1;
            if chunk.len() == chunk.capacity() {
                if !on_chunk(self.prime_index - chunk.len(), &chunk) {
                    return;
                }
                chunk.clear();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn calculates_the_first_10_primes_correctly() {
        let known_primes = vec![2, 3, 5, 7, 11, 13, 17, 19, 23, 29];
        assert_eq!(known_primes.len(), 10);

        let mut primer = Primer::new();

        let mut expected_index = 0;
        primer.crunch(1, |index, primes| {
            assert_eq!(index, expected_index);
            expected_index += 1;

            assert_eq!(primes.len(), 1);
            assert_eq!(
                primes.first().expect("primes should have a single entry"),
                &known_primes[index]
            );

            expected_index < known_primes.len()
        });

        assert_eq!(expected_index, known_primes.len());
    }

    #[test]
    fn calculates_the_first_1m_primes_correctly() {
        let mut primer = Primer::new();

        let file = "p1_000_000.txt";

        let known_primes = fs::read_to_string(file)
            .expect(format!("should be able to load {file}", file = file).as_str())
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.parse::<i64>())
            .collect::<Result<Vec<i64>, _>>()
            .expect(format!("should be able to parse {file}", file = file).as_str());
        assert_eq!(known_primes.len(), 1_000_000);

        let mut did_crunch = false;
        primer.crunch(known_primes.len(), |index, primes| {
            did_crunch = true;
            assert_eq!(index, 0);
            assert_eq!(primes.len(), known_primes.len());
            for i in 0..primes.len() {
                assert_eq!(
                    primes[i],
                    known_primes[i],
                    "Prime at index {i} differs! ({actual} != {expected})",
                    i = i,
                    actual = primes[i],
                    expected = known_primes[i]
                );
            }
            false
        });

        assert!(did_crunch);
    }
}
