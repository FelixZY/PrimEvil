package se.fzy.primevil

object Primer {
    init {
        System.loadLibrary("primer")
    }

    external fun crunch(n: Int): Long

    data class CrunchResult(val lastPrime: Long, val totalPrimes: Int)
}
