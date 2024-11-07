package se.fzy.primevil

import androidx.annotation.Keep

fun interface OnChunkListener {
    @Keep @Suppress("unused") fun onChunk(firstPrimeIndex: Int, chunk: LongArray): Boolean
}

object Primer {
    init {
        System.loadLibrary("primer")
    }

    @Keep external fun crunch(chunkSize: Int, listener: OnChunkListener)

    data class CrunchResult(val lastPrime: Long, val totalPrimes: Int)
}
