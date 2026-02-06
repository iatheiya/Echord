package app.vichord.android

import app.vichord.android.service.LOCAL_KEY_PREFIX
import app.vichord.core.data.enums.*

/**
 * Utilities for dynamic SQL clauses (use in @RawQuery).
 * Pre: Enums valid. Post: Safe SELECT clause (no injection, params separate).
 * Note: For 2025, consider compiled if frequent (Room supports).
 */
object DatabaseUtils {

    /**
     * Builds song sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * @param isLocal Local.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported sortBy.
     */
    fun buildSongSortClause(sortBy: SongSortBy, sortOrder: SortOrder, isLocal: Boolean = false): String {
        val baseWhere = if (isLocal) "WHERE id LIKE '$LOCAL_KEY_PREFIX%'" else "WHERE id NOT LIKE '$LOCAL_KEY_PREFIX%'"
        val orderClause = when (sortBy) {
            SongSortBy.Title -> "title COLLATE NOCASE ${sortOrder.suffix}"
            SongSortBy.PlayTime -> "totalPlayTimeMs ${sortOrder.suffix}"
            SongSortBy.DateAdded -> "ROWID ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported sortBy: $sortBy")
        }
        return "SELECT * FROM Song $baseWhere ORDER BY $orderClause"
    }

    private val SortOrder.suffix: String get() = if (this == SortOrder.Ascending) "ASC" else "DESC"

    /**
     * Builds favorites sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported.
     */
    fun buildFavoritesSortClause(sortBy: SongSortBy, sortOrder: SortOrder): String {
        val baseWhere = "WHERE likedAt IS NOT NULL"
        val orderClause = when (sortBy) {
            SongSortBy.Title -> "title COLLATE NOCASE ${sortOrder.suffix}"
            SongSortBy.PlayTime -> "totalPlayTimeMs ${sortOrder.suffix}"
            SongSortBy.DateAdded -> "likedAt ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported favorites sortBy: $sortBy")
        }
        return "SELECT * FROM Song $baseWhere ORDER BY $orderClause"
    }

    /**
     * Builds artists sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported.
     */
    fun buildArtistsSortClause(sortBy: ArtistSortBy, sortOrder: SortOrder): String {
        val baseWhere = "WHERE bookmarkedAt IS NOT NULL"
        val orderClause = when (sortBy) {
            ArtistSortBy.Name -> "name COLLATE NOCASE ${sortOrder.suffix}"
            ArtistSortBy.DateAdded -> "bookmarkedAt ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported artist sortBy: $sortBy")
        }
        return "SELECT * FROM Artist $baseWhere ORDER BY $orderClause"
    }

    /**
     * Builds albums sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported.
     */
    fun buildAlbumSortClause(sortBy: AlbumSortBy, sortOrder: SortOrder): String {
        val baseWhere = "WHERE bookmarkedAt IS NOT NULL"
        val orderClause = when (sortBy) {
            AlbumSortBy.Title -> "title COLLATE NOCASE ${sortOrder.suffix}"
            AlbumSortBy.Year -> "year \( {sortOrder.suffix}, authorsText COLLATE NOCASE \){sortOrder.suffix}"
            AlbumSortBy.DateAdded -> "bookmarkedAt ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported album sortBy: $sortBy")
        }
        return "SELECT * FROM Album $baseWhere ORDER BY $orderClause"
    }

    /**
     * Builds playlist previews sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported.
     */
    fun buildPlaylistPreviewsSortClause(sortBy: PlaylistSortBy, sortOrder: SortOrder): String {
        val baseWhere = "" // No where for previews
        val orderClause = when (sortBy) {
            PlaylistSortBy.Name -> "name COLLATE NOCASE ${sortOrder.suffix}"
            PlaylistSortBy.SongCount -> "songCount ${sortOrder.suffix}"
            PlaylistSortBy.DateAdded -> "ROWID ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported playlist sortBy: $sortBy")
        }
        return "SELECT id, name, (SELECT COUNT(*) FROM SongPlaylistMap WHERE playlistId = id) as songCount, thumbnail FROM Playlist ORDER BY $orderClause"
    }

    /**
     * Builds songs with content length sort clause.
     * @param sortBy Criterion.
     * @param sortOrder Order.
     * Pre: Non-null.
     * Post: Valid SQL.
     * @throws IllegalArgumentException if unsupported.
     */
    fun buildSongsWithContentLengthSortClause(sortBy: SongSortBy, sortOrder: SortOrder): String {
        val baseWhere = "JOIN Format ON id = songId WHERE contentLength IS NOT NULL"
        val orderClause = when (sortBy) {
            SongSortBy.Title -> "Song.title COLLATE NOCASE ${sortOrder.suffix}"
            SongSortBy.PlayTime -> "Song.totalPlayTimeMs ${sortOrder.suffix}"
            SongSortBy.DateAdded -> "Song.ROWID ${sortOrder.suffix}"
            else -> throw IllegalArgumentException("Unsupported content sortBy: $sortBy")
        }
        return "SELECT Song.*, contentLength FROM Song $baseWhere ORDER BY $orderClause"
    }
}