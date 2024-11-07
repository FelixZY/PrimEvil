use crate::queue::{PriorityQueue, StorageBackedI64PriorityQueue};
use crate::storage::SqlitePrioQueueDao;

const STEP_SIZE: &[i64] = &[2, 4, 2, 2];
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

    pub fn crunch<FCancelled, FOnPrime>(
        &mut self,
        mut is_active: FCancelled,
        mut on_prime: FOnPrime,
    ) -> ()
    where
        FCancelled: FnMut() -> bool,
        FOnPrime: FnMut(usize, i64) -> (),
    {
        while is_active() && self.prime_index < PREDEFINED_PRIMES.len() {
            let prime = PREDEFINED_PRIMES[self.prime_index];
            self.last_candidate = prime;

            // 2 and 5 are accounted for via STEP_SIZE
            if prime != 2 && prime != 5 {
                // Primes above 2 cannot be even numbers.
                // Multiply by 3 to optimize for this.
                self.false_candidates.insert((prime * 3, prime));
            }

            on_prime(self.prime_index, prime);
            self.prime_index += 1;
        }
        if !is_active() {
            return;
        }

        let mut lowest = self
            .false_candidates
            .peek()
            .map(|(false_candidate, _)| *false_candidate)
            .expect("there will always be false candidates");
        while is_active() {
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

            // Primes above 2 cannot be even numbers.
            // Multiply by 3 to optimize for this.
            self.false_candidates.insert((candidate * 3, candidate));

            on_prime(self.prime_index, candidate);
            self.prime_index += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use std::fs;

    #[test]
    fn primer_aborts_if_cancelled() {
        let mut primer = Primer::new();

        primer.crunch(|| false, |_, _| assert!(false, "callback should not run"));
    }

    #[test]
    fn calculates_the_first_3m_primes_correctly() {
        let mut primer = Primer::new();

        let file = "p3_000_000.txt";

        let prime_count = Cell::new(0usize);
        let known_primes = RefCell::new(
            fs::read_to_string(file)
                .expect(format!("should be able to load {file}", file = file).as_str())
                .lines()
                .filter(|l| !l.is_empty())
                .map(|l| l.parse::<i64>())
                .collect::<Result<Vec<i64>, _>>()
                .expect(format!("should be able to parse {file}", file = file).as_str()),
        );

        primer.crunch(
            || prime_count.get() < known_primes.borrow().len(),
            |index, prime| {
                assert_eq!(prime_count.get(), index);
                assert_eq!(prime, *known_primes.borrow().get(index).unwrap());

                prime_count.set(prime_count.get() + 1);
            },
        );

        assert_eq!(prime_count.get(), known_primes.borrow().len());
    }
}
