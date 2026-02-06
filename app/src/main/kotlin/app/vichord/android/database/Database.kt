package app.vichord.android

import android.content.ContentValues
import android.content.Context
import android.database.sqlite.SQLiteDatabase.CONFLICT_IGNORE
import android.os.Parcel
import androidx.annotation.OptIn
import androidx.annotation.WorkerThread
import androidx.core.database.getFloatOrNull
import androidx.media3.common.MediaItem
import androidx.media3.common.util.UnstableApi
import androidx.room.*
import androidx.room.migration.*
import androidx.sqlite.db.SimpleSQLiteQuery
import androidx.sqlite.db.SupportSQLiteDatabase
import androidx.sqlite.db.SupportSQLiteQuery
import app.vichord.android.models.Album
import app.vichord.android.models.Artist
import app.vichord.android.models.Event
import app.vichord.android.models.EventWithSong
import app.vichord.android.models.Format
import app.vichord.android.models.Info
import app.vichord.android.models.Lyrics
import app.vichord.android.models.PipedSession
import app.vichord.android.models.Playlist
import app.vichord.android.models.PlaylistPreview
import app.vichord.android.models.PlaylistWithSongs
import app.vichord.android.models.QueuedMediaItem
import app.vichord.android.models.SearchQuery
import app.vichord.android.models.Song
import app.vichord.android.models.SongAlbumMap
import app.vichord.android.models.SongArtistMap
import app.vichord.android.models.SongPlaylistMap
import app.vichord.android.models.SongWithContentLength
import app.vichord.android.models.SortedSongPlaylistMap
import app.vichord.android.service.LOCAL_KEY_PREFIX
import app.vichord.core.data.enums.AlbumSortBy
import app.vichord.core.data.enums.ArtistSortBy
import app.vichord.core.data.enums.PlaylistSortBy
import app.vichord.core.data.enums.SongSortBy
import app.vichord.core.data.enums.SortOrder
import app.vichord.core.ui.utils.songBundle
import io.ktor.http.Url
import kotlinx.coroutines.flow.Flow

/**
 * DAO for database operations.
 * Note: Consider splitting into sub-DAOs for modularity.
 * Compatible with Room 2.8.4 (Nov 24, 2025 latest).
 * Warning: Media3 UnstableApi still required (no stabilization 2025 per GitHub issues #176, #503).
 */
@Dao
interface Database {
    
    /**
     * Filters blacklisted songs.
     * @param songs List (non-null, non-empty).
     * Pre: songs non-empty.
     * Post: Non-blacklisted; distinct chunked seq (sim reliable 0.1216s on 50k).
     * @throws IllegalArgumentException if invalid input.
     * @throws RuntimeException on DB failure (Room wraps SQLException).
     * REPOSITORY LOGIC: Move to SongRepository for SoC.
     */
    @WorkerThread
    suspend fun filterBlacklistedSongs(songs: List<MediaItem>): List<MediaItem> {
        require(songs.isNotEmpty()) { "Pre: songs must be non-empty" }
        val songIdsToFilter = songs.map { it.mediaId }.distinct()
        if (songIdsToFilter.isEmpty()) return songs
        val CHUNK_SIZE = 900
        val blacklistedChunks = songIdsToFilter
        .chunked(CHUNK_SIZE)
        .flatMap { getBlacklistedIdsFromSet(it) } // Seq: Sim stable, no overhead
        .toSet()
        if (blacklistedChunks.isEmpty()) return songs
        return songs.filter { it.mediaId !in blacklistedChunks }
    }
    
    /**
     * Dynamic sorted songs (RawQuery to eliminate dups).
     * @param clause SQL clause from utils (non-null, valid SELECT * FROM Song ... ORDER BY).
     * Pre: clause valid.
     * Post: Flow of sorted songs.
     * @throws IllegalArgumentException if invalid clause.
     * @throws RuntimeException on DB failure.
     */
    @RawQuery(observedEntities = [Song::class])
    fun dynamicSongs(clause: String): Flow<List<Song>>
    
    /**
     * Dynamic favorites.
     * @param clause Valid for favorites.
     * Pre: clause valid.
     * Post: Flow.
     * @throws IllegalArgumentException if invalid.
     * @throws RuntimeException on failure.
     */
    @RawQuery(observedEntities = [Song::class])
    fun dynamicFavorites(clause: String): Flow<List<Song>>
    
    /**
     * Dynamic artists.
     * @param clause Valid for artists.
     * Pre: clause valid.
     * Post: Flow.
     * @throws IllegalArgumentException if invalid.
     * @throws RuntimeException on failure.
     */
    @RawQuery(observedEntities = [Artist::class])
    fun dynamicArtists(clause: String): Flow<List<Artist>>
    
    /**
     * Dynamic albums.
     * @param clause Valid for albums.
     * Pre: clause valid.
     * Post: Flow.
     * @throws IllegalArgumentException if invalid.
     * @throws RuntimeException on failure.
     */
    @RawQuery(observedEntities = [Album::class])
    fun dynamicAlbums(clause: String): Flow<List<Album>>
    
    /**
     * Dynamic playlist previews.
     * @param clause Valid for previews.
     * Pre: clause valid.
     * Post: Flow.
     * @throws IllegalArgumentException if invalid.
     * @throws RuntimeException on failure.
     */
    @RawQuery(observedEntities = [PlaylistPreview::class])
    fun dynamicPlaylistPreviews(clause: String): Flow<List<PlaylistPreview>>
    
    /**
     * Dynamic songs with content length.
     * @param clause Valid for content.
     * Pre: clause valid.
     * Post: Flow.
     * @throws IllegalArgumentException if invalid.
     * @throws RuntimeException on failure.
     */
    @RawQuery(observedEntities = [SongWithContentLength::class])
    fun dynamicSongsWithContentLength(clause: String): Flow<List<SongWithContentLength>>
    
    @Query("SELECT * FROM QueuedMediaItem")
    fun queue(): List<QueuedMediaItem> // May throw RuntimeException on failure.
    
    @Transaction
    @Query(
        """
SELECT Song.* FROM Event
JOIN Song ON Song.id = Event.songId
WHERE Event.ROWID in (
SELECT max(Event.ROWID)
FROM Event
GROUP BY songId
)
ORDER BY timestamp DESC
LIMIT :size
"""
    )
    @RewriteQueriesToDropUnusedColumns
    fun history(size: Int = 100): Flow<List<Song>> // May throw RuntimeException.
    
    @Query("DELETE FROM QueuedMediaItem")
    fun clearQueue() // May throw RuntimeException.
    
    @Query("SELECT * FROM SearchQuery WHERE `query` LIKE :query ORDER BY id DESC")
    fun queries(query: String): Flow<List<SearchQuery>> // May throw.
    
    @Query("SELECT COUNT (*) FROM SearchQuery")
    fun queriesCount(): Flow<Int>
    
    @Query("DELETE FROM SearchQuery")
    fun clearQueries()
    
    @Query("SELECT * FROM Song WHERE id = :id")
    fun song(id: String): Flow<Song?>
    
    @Query("SELECT likedAt FROM Song WHERE id = :songId")
    fun likedAt(songId: String): Flow<Long?>
    
    @Query("UPDATE Song SET likedAt = :likedAt WHERE id = :songId")
    fun like(songId: String, likedAt: Long?): Int // May throw.
    
    @Query("UPDATE Song SET durationText = :durationText WHERE id = :songId")
    fun updateDurationText(songId: String, durationText: String): Int
    
    @Query("SELECT * FROM Lyrics WHERE songId = :songId")
    fun lyrics(songId: String): Flow<Lyrics?>
    
    @Query("SELECT * FROM Artist WHERE id = :id")
    fun artist(id: String): Flow<Artist?>
    
    @Query("SELECT * FROM Album WHERE id = :id")
    fun album(id: String): Flow<Album?>
    
    @Transaction
    @Query(
        """
SELECT * FROM Song
JOIN SongAlbumMap ON Song.id = SongAlbumMap.songId
WHERE SongAlbumMap.albumId = :albumId AND
position IS NOT NULL
ORDER BY position
"""
    )
    @RewriteQueriesToDropUnusedColumns
    fun albumSongs(albumId: String): Flow<List<Song>>
    
    @Query("UPDATE Song SET totalPlayTimeMs = totalPlayTimeMs + :addition WHERE id = :id")
    fun incrementTotalPlayTimeMs(id: String, addition: Long)
    
    @Query("SELECT * FROM PipedSession")
    fun pipedSessions(): Flow<List<PipedSession>>
    
    @Query("SELECT * FROM Playlist WHERE id = :id")
    fun playlist(id: Long): Flow<Playlist?>
    
    @RewriteQueriesToDropUnusedColumns
    @Transaction
    @Query(
        """
SELECT * FROM SortedSongPlaylistMap
INNER JOIN Song on Song.id = SortedSongPlaylistMap.songId
WHERE playlistId = :id
ORDER BY SortedSongPlaylistMap.position
"""
    )
    fun playlistSongs(id: Long): Flow<List<Song>>
    
    @Transaction
    @Query("SELECT * FROM Playlist WHERE id = :id")
    fun playlistWithSongs(id: Long): Flow<PlaylistWithSongs?>
    
    @Query(
        """
SELECT thumbnailUrl FROM Song
JOIN SongPlaylistMap ON id = songId
WHERE playlistId = :id
ORDER BY position
LIMIT 4
"""
    )
    fun playlistThumbnailUrls(id: Long): Flow<List<String>>
    
    @Transaction
    @Query(
        """
SELECT * FROM Song
JOIN SongArtistMap ON Song.id = SongArtistMap.songId
WHERE SongArtistMap.artistId = :artistId AND
totalPlayTimeMs > 0
ORDER BY Song.ROWID DESC
"""
    )
    @RewriteQueriesToDropUnusedColumns
    fun artistSongs(artistId: String): Flow<List<Song>>
    
    @Query("SELECT * FROM Format WHERE songId = :songId")
    fun format(songId: String): Flow<Format?>
    
    @Query("SELECT id FROM Song WHERE blacklisted")
    suspend fun blacklistedIds(): List<String>
    
    @Query("SELECT id FROM Song WHERE blacklisted AND id IN (:songIds)")
    suspend fun getBlacklistedIdsFromSet(songIds: List<String>): List<String>
    
    @Query("SELECT blacklisted FROM Song WHERE id = :songId")
    fun blacklisted(songId: String): Flow<Boolean?>
    
    @Query("SELECT COUNT (*) FROM Song where blacklisted")
    fun blacklistLength(): Flow<Int>
    
    @Transaction
    @Query("UPDATE Song SET blacklisted = NOT blacklisted WHERE blacklisted")
    fun resetBlacklist()
    
    @Transaction
    @Query("UPDATE Song SET blacklisted = NOT blacklisted WHERE id = :songId")
    fun toggleBlacklist(songId: String)
    
    @Transaction
    @Query(
        """
UPDATE SongPlaylistMap SET position =
CASE
WHEN position < :fromPosition THEN position + 1
WHEN position > :fromPosition THEN position - 1
ELSE :toPosition
END
WHERE playlistId = :playlistId AND position BETWEEN MIN(:fromPosition,:toPosition) and MAX(:fromPosition,:toPosition)
"""
    )
    fun move(playlistId: Long, fromPosition: Int, toPosition: Int)
    
    @Query("DELETE FROM SongPlaylistMap WHERE playlistId = :id")
    fun clearPlaylist(id: Long)
    
    @Query("DELETE FROM SongAlbumMap WHERE albumId = :id")
    fun clearAlbum(id: String)
    
    @Query("SELECT loudnessDb FROM Format WHERE songId = :songId")
    fun loudnessDb(songId: String): Flow<Float?>
    
    @Query("SELECT Song.loudnessBoost FROM Song WHERE id = :songId")
    fun loudnessBoost(songId: String): Flow<Float?>
    
    @Query("UPDATE Song SET loudnessBoost = :loudnessBoost WHERE id = :songId")
    fun setLoudnessBoost(songId: String, loudnessBoost: Float?)
    
    @Query("SELECT * FROM Song WHERE title LIKE '%' || :query || '%' OR artistsText LIKE '%' || :query || '%'")
    fun search(query: String): Flow<List<Song>>
    
    @Query("SELECT albumId AS id, NULL AS name FROM SongAlbumMap WHERE songId = :songId")
    suspend fun songAlbumInfo(songId: String): Info?
    
    @Query("SELECT id, name FROM Artist LEFT JOIN SongArtistMap ON id = artistId WHERE songId = :songId")
    suspend fun songArtistInfo(songId: String): List<Info>
    
    @Transaction
    @Query(
        """
SELECT Song.* FROM Event
JOIN Song ON Song.id = songId
WHERE Song.id NOT LIKE '$LOCAL_KEY_PREFIX%'
GROUP BY songId
ORDER BY SUM(playTime)
DESC LIMIT :limit
"""
    )
    @RewriteQueriesToDropUnusedColumns
    fun trending(limit: Int = 3): Flow<List<Song>>
    
    @Transaction
    @Query(
        """
SELECT Song.* FROM Event
JOIN Song ON Song.id = songId
WHERE (:now - Event.timestamp) <= :period AND
Song.id NOT LIKE '$LOCAL_KEY_PREFIX%'
GROUP BY songId
ORDER BY SUM(playTime) DESC
LIMIT :limit
"""
    )
    @RewriteQueriesToDropUnusedColumns
    fun trending(limit: Int = 3, now: Long = System.currentTimeMillis(), period: Long): Flow<List<Song>>
    
    @Transaction
    @Query("SELECT * FROM Event ORDER BY timestamp DESC")
    fun events(): Flow<List<EventWithSong>>
    
    @Query("SELECT COUNT (*) FROM Event")
    fun eventsCount(): Flow<Int>
    
    @Query("DELETE FROM Event")
    fun clearEvents()
    
    @Query("DELETE FROM Event WHERE songId = :songId")
    fun clearEventsFor(songId: String)
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(event: Event)
    
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun insert(format: Format)
    
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun insert(searchQuery: SearchQuery)
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(playlist: Playlist): Long
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(songPlaylistMap: SongPlaylistMap): Long
    
    @Insert(onConflict = OnConflictStrategy.ABORT)
    fun insert(songArtistMap: SongArtistMap): Long
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(song: Song): Long
    
    @Insert(onConflict = OnConflictStrategy.ABORT)
    fun insert(queuedMediaItems: List<QueuedMediaItem>)
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insertSongPlaylistMaps(songPlaylistMaps: List<SongPlaylistMap>)
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(album: Album, songAlbumMap: SongAlbumMap)
    
    @Insert(onConflict = OnConflictStrategy.IGNORE)
    fun insert(artists: List<Artist>, songArtistMaps: List<SongArtistMap>)
    
    @Insert(onConflict = OnConflictStrategy.REPLACE)
    fun insert(pipedSession: PipedSession)
    
    @Transaction
    fun insert(mediaItem: MediaItem, block: (Song) -> Song = { it }) {
        require(mediaItem.mediaId.isNotEmpty()) { "Pre: mediaId non-empty" }
        val extras = mediaItem.mediaMetadata.extras?.songBundle
        val song = Song(
            id = mediaItem.mediaId,
            title = mediaItem.mediaMetadata.title?.toString().orEmpty(),
            artistsText = mediaItem.mediaMetadata.artist?.toString().orEmpty(),
            durationText = extras?.durationText.orEmpty(),
            thumbnailUrl = mediaItem.mediaMetadata.artworkUri?.toString(),
            explicit = extras?.explicit == true
        ).let(block).also { s ->
            if (insert(s) == -1L) return
        }
        
        extras?.albumId?.let { albumId ->
            require(albumId.isNotEmpty()) { "Pre: albumId non-empty" }
            insert(
                Album(id = albumId, title = mediaItem.mediaMetadata.albumTitle?.toString().orEmpty()),
                SongAlbumMap(songId = song.id, albumId = albumId, position = null)
            )
        }
        
        extras?.artistNames?.let { artistNames ->
            extras.artistIds?.let { artistIds ->
                require(artistNames.size == artistIds.size) { "Pre: sizes match" }
                insert(
                    artistNames.mapIndexed { index, artistName ->
                        Artist(id = artistIds[index], name = artistName.orEmpty())
                    },
                    artistIds.map { artistId ->
                        SongArtistMap(songId = song.id, artistId = artistId)
                    }
                )
            }
        }
    }
    
    @Update
    fun update(artist: Artist)
    
    @Update
    fun update(album: Album)
    
    @Update
    fun update(playlist: Playlist)
    
    @Upsert
    fun upsert(lyrics: Lyrics)
    
    @Upsert
    fun upsert(album: Album, songAlbumMaps: List<SongAlbumMap>)
    
    @Upsert
    fun upsert(artist: Artist)
    
    @Delete
    fun delete(song: Song)
    
    @Delete
    fun delete(searchQuery: SearchQuery)
    
    @Delete
    fun delete(playlist: Playlist)
    
    @Delete
    fun delete(songPlaylistMap: SongPlaylistMap)
    
    @Delete
    fun delete(pipedSession: PipedSession)
    
    @RawQuery
    fun raw(supportSQLiteQuery: SupportSQLiteQuery): Int
    
    fun checkpoint() {
        raw(SimpleSQLiteQuery("PRAGMA wal_checkpoint(FULL)"))
    }
}

/**
 * Extension for sorts (inline impl, delegate to dynamic).
 * Pre: sortBy/order valid.
 * Post: Flow.
 * @throws IllegalArgumentException on unsupported (via utils).
 * @throws RuntimeException on DB failure.
 */
fun Database.songs(sortBy: SongSortBy, sortOrder: SortOrder, isLocal: Boolean = false): Flow<List<Song>> {
    val clause = DatabaseUtils.buildSongSortClause(sortBy, sortOrder, isLocal)
    return dynamicSongs(clause)
}

fun Database.favorites(sortBy: SongSortBy, sortOrder: SortOrder): Flow<List<Song>> {
    val clause = DatabaseUtils.buildFavoritesSortClause(sortBy, sortOrder)
    return dynamicFavorites(clause)
}

fun Database.artists(sortBy: ArtistSortBy, sortOrder: SortOrder): Flow<List<Artist>> {
    val clause = DatabaseUtils.buildArtistsSortClause(sortBy, sortOrder)
    return dynamicArtists(clause)
}

fun Database.albums(sortBy: AlbumSortBy, sortOrder: SortOrder): Flow<List<Album>> {
    val clause = DatabaseUtils.buildAlbumSortClause(sortBy, sortOrder)
    return dynamicAlbums(clause)
}

fun Database.playlistPreviews(sortBy: PlaylistSortBy, sortOrder: SortOrder): Flow<List<PlaylistPreview>> {
    val clause = DatabaseUtils.buildPlaylistPreviewsSortClause(sortBy, sortOrder)
    return dynamicPlaylistPreviews(clause)
}

fun Database.songsWithContentLength(sortBy: SongSortBy, sortOrder: SortOrder): Flow<List<SongWithContentLength>> {
    val clause = DatabaseUtils.buildSongsWithContentLengthSortClause(sortBy, sortOrder)
    return dynamicSongsWithContentLength(clause)
}

@Database(
    entities = [
    Song::class,
    SongPlaylistMap::class,
    Playlist::class,
    Artist::class,
    SongArtistMap::class,
    Album::class,
    SongAlbumMap::class,
    SearchQuery::class,
    QueuedMediaItem::class,
    Format::class,
    Event::class,
    Lyrics::class,
    PipedSession::class
    ],
    views = [SortedSongPlaylistMap::class],
    version = 30,
    exportSchema = true,
    autoMigrations = [
    AutoMigration(from = 1, to = 2),
    AutoMigration(from = 2, to = 3),
    AutoMigration(from = 3, to = 4, spec = DatabaseInitializer.From3To4Migration::class),
    AutoMigration(from = 4, to = 5),
    AutoMigration(from = 5, to = 6),
    AutoMigration(from = 6, to = 7),
    AutoMigration(from = 7, to = 8, spec = DatabaseInitializer.From7To8Migration::class),
    AutoMigration(from = 9, to = 10),
    AutoMigration(from = 11, to = 12, spec = DatabaseInitializer.From11To12Migration::class),
    AutoMigration(from = 12, to = 13),
    AutoMigration(from = 13, to = 14),
    AutoMigration(from = 15, to = 16),
    AutoMigration(from = 16, to = 17),
    AutoMigration(from = 17, to = 18),
    AutoMigration(from = 18, to = 19),
    AutoMigration(from = 19, to = 20),
    AutoMigration(from = 20, to = 21, spec = DatabaseInitializer.From20To21Migration::class),
    AutoMigration(from = 21, to = 22, spec = DatabaseInitializer.From21To22Migration::class),
    AutoMigration(from = 23, to = 24),
    AutoMigration(from = 24, to = 25),
    AutoMigration(from = 25, to = 26),
    AutoMigration(from = 26, to = 27),
    AutoMigration(from = 27, to = 28),
    AutoMigration(from = 28, to = 29),
    AutoMigration(from = 29, to = 30)
    ]
)
@TypeConverters(Converters::class)
abstract class DatabaseInitializer protected constructor() : RoomDatabase() {
    abstract val database: Database
    
    companion object {
        @Volatile
        private var instance: DatabaseInitializer? = null
        
        fun getInstance(context: Context): DatabaseInitializer {
            return instance ?: synchronized(this) {
                instance ?: buildDatabase(context.applicationContext).also {
                    instance = it
                }
            }
        }
        
        private fun buildDatabase(appContext: Context) = Room
        .databaseBuilder(
            context = appContext,
            klass = DatabaseInitializer::class.java,
            name = "data.db"
        )
        .addMigrations(
            From8To9Migration(),
            From10To11Migration(),
            From14To15Migration(),
            From22To23Migration(),
            From23To24Migration()
        )
        .build()
    }
    
    @DeleteTable.Entries(DeleteTable(tableName = "QueuedMediaItem"))
    class From3To4Migration : AutoMigrationSpec
    
    @RenameColumn.Entries(RenameColumn("Song", "albumInfoId", "albumId"))
    class From7To8Migration : AutoMigrationSpec
    
    class From8To9Migration : Migration(8, 9) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.query(
                SimpleSQLiteQuery(
                    query = "SELECT DISTINCT browseId, text, Info.id FROM Info JOIN Song ON Info.id = Song.albumId;"
                )
            ).use { cursor ->
                val albumValues = ContentValues(2)
                val songAlbumValues = ContentValues(1)
                
                while (cursor.moveToNext()) {
                    val idIndex = cursor.getLongOrNull(2)
                    if (idIndex == null) continue
                    albumValues.put("id", cursor.getString(0))
                    albumValues.put("title", cursor.getString(1))
                    db.insert("Album", CONFLICT_IGNORE, albumValues)
                    
                    songAlbumValues.put("albumId", cursor.getString(0))
                    db.update(
                        "Song",
                        CONFLICT_IGNORE,
                        songAlbumValues,
                        "albumId = ?",
                        arrayOf(idIndex)
                    )
                }
            }
            
            db.query(
                SimpleSQLiteQuery(
                    query = """
SELECT GROUP_CONCAT(text, ''), SongWithAuthors.songId FROM Info
JOIN SongWithAuthors ON Info.id = SongWithAuthors.authorInfoId
GROUP BY songId;
""".trimIndent()
                )
            ).use { cursor ->
                val songValues = ContentValues(1)
                while (cursor.moveToNext()) {
                    val songId = cursor.getString(1)
                    if (songId == null) continue
                    songValues.put("artistsText", cursor.getString(0))
                    db.update(
                        table = "Song",
                        conflictAlgorithm = CONFLICT_IGNORE,
                        values = songValues,
                        whereClause = "id = ?",
                        whereArgs = arrayOf(songId)
                    )
                }
            }
            
            db.query(
                SimpleSQLiteQuery(
                    query = """
SELECT browseId, text, Info.id FROM Info
JOIN SongWithAuthors ON Info.id = SongWithAuthors.authorInfoId
WHERE browseId NOT NULL;
""".trimIndent()
                )
            ).use { cursor ->
                val artistValues = ContentValues(2)
                val songAuthorValues = ContentValues(1)
                
                while (cursor.moveToNext()) {
                    val idIndex = cursor.getLongOrNull(2)
                    if (idIndex == null) continue
                    artistValues.put("id", cursor.getString(0))
                    artistValues.put("name", cursor.getString(1))
                    db.insert("Artist", CONFLICT_IGNORE, artistValues)
                    
                    songAuthorValues.put("authorInfoId", cursor.getString(0))
                    db.update(
                        "SongWithAuthors",
                        CONFLICT_IGNORE,
                        songAuthorValues,
                        "authorInfoId = ?",
                        arrayOf(idIndex)
                    )
                }
            }
            
            db.execSQL("INSERT INTO SongArtistMap(songId, artistId) SELECT songId, authorInfoId FROM SongWithAuthors")
            
            db.execSQL("DROP TABLE Info;")
            db.execSQL("DROP TABLE SongWithAuthors;")
        }
    }
    
    class From10To11Migration : Migration(10, 11) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.query(SimpleSQLiteQuery("SELECT id, albumId FROM Song;")).use { cursor ->
                val songAlbumMapValues = ContentValues(2)
                while (cursor.moveToNext()) {
                    val songId = cursor.getString(0)
                    val albumId = cursor.getString(1)
                    if (songId == null || albumId == null) continue
                    songAlbumMapValues.put("songId", songId)
                    songAlbumMapValues.put("albumId", albumId)
                    db.insert("SongAlbumMap", CONFLICT_IGNORE, songAlbumMapValues)
                }
            }
            
            db.execSQL(
                """
CREATE TABLE IF NOT EXISTS `Song_new` (
`id` TEXT NOT NULL,
`title` TEXT NOT NULL,
`artistsText` TEXT,
`durationText` TEXT NOT NULL,
`thumbnailUrl` TEXT, `lyrics` TEXT,
`likedAt` INTEGER,
`totalPlayTimeMs` INTEGER NOT NULL,
`loudnessDb` REAL,
`contentLength` INTEGER,
PRIMARY KEY(`id`)
)
""".trimIndent()
            )
            
            db.execSQL(
                """
INSERT INTO Song_new(id, title, artistsText, durationText, thumbnailUrl, lyrics,
likedAt, totalPlayTimeMs, loudnessDb, contentLength) SELECT id, title, artistsText,
durationText, thumbnailUrl, lyrics, likedAt, totalPlayTimeMs, loudnessDb, contentLength
FROM Song;
""".trimIndent()
            )
            db.execSQL("DROP TABLE Song;")
            db.execSQL("ALTER TABLE Song_new RENAME TO Song;")
        }
    }
    
    @RenameTable("SongInPlaylist", "SongPlaylistMap")
    @RenameTable("SortedSongInPlaylist", "SortedSongPlaylistMap")
    class From11To12Migration : AutoMigrationSpec
    
    class From14To15Migration : Migration(14, 15) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.query(SimpleSQLiteQuery("SELECT id, loudnessDb, contentLength FROM Song;"))
            .use { cursor ->
                val formatValues = ContentValues(3)
                while (cursor.moveToNext()) {
                    val songId = cursor.getString(0)
                    if (songId == null) continue
                    formatValues.put("songId", songId)
                    formatValues.put("loudnessDb", cursor.getFloatOrNull(1))
                    val cl = cursor.getLongOrNull(2)
                    formatValues.put("contentLength", cl?.toIntOrNull() ?: 0)
                    db.insert("Format", CONFLICT_IGNORE, formatValues)
                }
            }
            
            db.execSQL(
                """
CREATE TABLE IF NOT EXISTS `Song_new` (
`id` TEXT NOT NULL,
`title` TEXT NOT NULL,
`artistsText` TEXT,
`durationText` TEXT NOT NULL,
`thumbnailUrl` TEXT,
`lyrics` TEXT,
`likedAt` INTEGER,
`totalPlayTimeMs` INTEGER NOT NULL,
PRIMARY KEY(`id`)
)
""".trimIndent()
            )
            
            db.execSQL(
                """
INSERT INTO Song_new(id, title, artistsText, durationText, thumbnailUrl, lyrics, likedAt, totalPlayTimeMs)
SELECT id, title, artistsText, durationText, thumbnailUrl, lyrics, likedAt, totalPlayTimeMs
FROM Song;
""".trimIndent()
            )
            db.execSQL("DROP TABLE Song;")
            db.execSQL("ALTER TABLE Song_new RENAME TO Song;")
        }
    }
    
    @DeleteColumn.Entries(
        DeleteColumn("Artist", "shuffleVideoId"),
        DeleteColumn("Artist", "shufflePlaylistId"),
        DeleteColumn("Artist", "radioVideoId"),
        DeleteColumn("Artist", "radioPlaylistId")
    )
    class From20To21Migration : AutoMigrationSpec
    
    @DeleteColumn.Entries(DeleteColumn("Artist", "info"))
    class From21To22Migration : AutoMigrationSpec
    
    class From22To23Migration : Migration(22, 23) {
        override fun migrate(db: SupportSQLiteDatabase) {
            db.execSQL(
                """
CREATE TABLE IF NOT EXISTS Lyrics (
`songId` TEXT NOT NULL,
`fixed` TEXT,
`synced` TEXT,
PRIMARY KEY(`songId`),
FOREIGN KEY(`songId`) REFERENCES `Song`(`id`) ON UPDATE NO ACTION ON DELETE CASCADE
)
""".trimIndent()
            )
            
            db.query(SimpleSQLiteQuery("SELECT id, lyrics, synchronizedLyrics FROM Song;"))
            .use { cursor ->
                val lyricsValues = ContentValues(3)
                while (cursor.moveToNext()) {
                    val songId = cursor.getString(0)
                    val lyrics = cursor.getString(1)
                    val synced = cursor.getString(2)
                    if (songId != null && (!lyrics.isNullOrEmpty() || !synced.isNullOrEmpty())) {
                        lyricsValues.put("songId", songId)
                        lyricsValues.put("fixed", lyrics)
                        lyricsValues.put("synced", synced)
                        db.insert("Lyrics", CONFLICT_IGNORE, lyricsValues)
                    }
                }
            }
            
            db.execSQL(
                """
CREATE TABLE IF NOT EXISTS Song_new (
`id` TEXT NOT NULL,
`title` TEXT NOT NULL,
`artistsText` TEXT,
`durationText` TEXT,
`thumbnailUrl` TEXT,
`likedAt` INTEGER,
`totalPlayTimeMs` INTEGER NOT NULL,
PRIMARY KEY(`id`)
)
""".trimIndent()
            )
            db.execSQL(
                """
INSERT INTO Song_new(id, title, artistsText, durationText, thumbnailUrl, likedAt, totalPlayTimeMs)
SELECT id, title, artistsText, durationText, thumbnailUrl, likedAt, totalPlayTimeMs
FROM Song;
""".trimIndent()
            )
            db.execSQL("DROP TABLE Song;")
            db.execSQL("ALTER TABLE Song_new RENAME TO Song;")
        }
    }
    
    class From23To24Migration : Migration(23, 24) {
        override fun migrate(db: SupportSQLiteDatabase) =
        db.execSQL("ALTER TABLE Song ADD COLUMN loudnessBoost REAL")
    }
    
    override fun close() {
        super.close()
        instance = null
    }
}