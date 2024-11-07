package se.fzy.primevil

import java.util.TreeMap
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

private val primeStepArray = arrayOf(2, 4, 2, 2)

@Deprecated("Use native implementation instead.")
class PrimerOld {
    val falseCandidateCache = TreeMap<Long, MutableList<Long>>()
    private val candidateSequence: Sequence<Long> =
        sequenceOf<Long>(2, 3, 5, 7, 11) +
            sequence {
                @Suppress("MagicNumber") var candidate = 11L
                var stepIndex = 0
                while (true) {
                    candidate += primeStepArray[stepIndex]
                    stepIndex = (stepIndex + 1) % primeStepArray.size
                    yield(candidate)
                }
            }

    var primeMachineGeneratedCount = 0
        private set

    val primeMachine: Iterator<Long> =
        candidateSequence
            .filter { candidate ->
                while (
                    falseCandidateCache.isNotEmpty() && candidate >= falseCandidateCache.firstKey()
                ) {
                    val (key, primes) = falseCandidateCache.pollFirstEntry()!!
                    primes.forEachIndexed { i, prime ->
                        // The sum of two odd numbers are always even
                        val falseCandidate = key + prime * 2
                        falseCandidateCache
                            .getOrPut(falseCandidate) {
                                // Reuse existing arrays where possible
                                if (i + 1 == primes.size) primes.apply { clear() }
                                else ArrayList(guessCapacityForCandidate(falseCandidate))
                            }
                            .add(prime)
                    }
                    falseCandidateCache.remove(key)
                    if (key == candidate) {
                        return@filter false
                    }
                }

                // We are already optimizing these away via primeStepArray
                if (candidate != 2L && candidate != 5L) {
                    // Primes can only be odd.
                    @Suppress("MagicNumber") val falseCandidate = candidate * 3L
                    falseCandidateCache
                        .getOrPut(falseCandidate) {
                            ArrayList(guessCapacityForCandidate(falseCandidate))
                        }
                        .add(candidate)
                }
                primeMachineGeneratedCount++
                return@filter true
            }
            .iterator()

    suspend fun crunch(limit: Int = -1, onEach: suspend (CrunchResult) -> Unit = {}): CrunchResult =
        withContext(Dispatchers.IO) {
            var lastPrime = -1L
            var totalPrimes = 0
            while (isActive && (limit < 0 || totalPrimes < limit)) {
                lastPrime = primeMachine.next()
                totalPrimes++
                launch { onEach(CrunchResult(lastPrime, totalPrimes)) }
            }
            return@withContext CrunchResult(lastPrime, totalPrimes)
        }

    data class CrunchResult(val lastPrime: Long, val totalPrimes: Int)

    @Suppress("MagicNumber")
    private fun guessCapacityForCandidate(n: Long) =
        // Allocating arrays anew are expensive. Testing indicates that these sizes should result in
        // minimal allocations.
        when (n) {
            in 0L..9L -> 2
            in 10L..99L -> 3
            in 100L..999L -> 4
            in 1000L..9999L -> 5
            in 10000L..99999L -> 6
            in 100000L..999999L -> 7
            in 1000000L..9999999L -> 8
            else -> 9
        }
}
