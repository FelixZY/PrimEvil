package se.fzy.primevil.primer

object Native {
    init {
        System.loadLibrary("primer")
    }

    external fun add(a: Int, b: Int): Int
}
