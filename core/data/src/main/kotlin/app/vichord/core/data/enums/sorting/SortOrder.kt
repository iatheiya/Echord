package app.vitune.core.data.enums.sorting

enum class SortOrder {
    Ascending,
    Descending;

    operator fun not() = when (this) {
        Ascending -> Descending
        Descending -> Ascending
    }
}
