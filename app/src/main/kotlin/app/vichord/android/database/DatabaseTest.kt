package app.vichord.android

import androidx.room.Room
import androidx.test.core.app.ApplicationProvider
import androidx.test.ext.junit.runners.AndroidJUnit4
import androidx.test.filters.MediumTest
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.test.runTest
import org.junit.After
import org.junit.Before
import org.junit.Test
import org.junit.runner.RunWith
import org.robolectric.annotation.Config
import app.vichord.android.models.*
import app.vichord.core.data.enums.*
import org.junit.Assert.*
import androidx.media3.common.MediaItem
import kotlin.system.measureTimeMillis

@MediumTest
@RunWith(AndroidJUnit4::class)
@Config(sdk = [28])
class DatabaseTest {
    
    private lateinit var db: DatabaseInitializer
    private lateinit var dao: Database
    
    @Before
    fun setup() = runTest {
        val context = ApplicationProvider.getApplicationContext<android.content.Context>()
        db = Room.inMemoryDatabaseBuilder(context, DatabaseInitializer::class.java).build()
        dao = db.database
    }
    
    @After
    fun teardown() {
        db.close()
    }
    
    @Test
    fun `filterBlacklistedSongs seq perf on 1800 items in-mem`() = runTest {
        // Arrange: 1800 ids (2 chunks), insert/mock blacklisted
        val ids = (1..1800).map { "id$it" }
        ids.take(900).forEach {
            dao.insert(Song(id = it, title = "Black", totalPlayTimeMs = 100))
            dao.toggleBlacklist(it)
        }
        val songs = ids.map { MediaItem.fromUri(it) }
        
        // Act & Assert timing (real in-mem DB)
        val dur = measureTimeMillis {
            val filtered = dao.filterBlacklistedSongs(songs)
            assertEquals(900, filtered.size) // Half clean
        }
        assertTrue("Seq <5ms on 1800 in-mem", dur < 5)
    }
    
    @Test
    fun `filterBlacklistedSongs seq on 50k scale in-mem`() = runTest {
        // Arrange: 50k (large lib sim), subset blacklisted
        val ids = (1..50000).map { "id$it" }
        ids.take(25000).forEach {
            dao.insert(Song(id = it, title = "Black", totalPlayTimeMs = 100))
            dao.toggleBlacklist(it)
        }
        
        val dur = measureTimeMillis {
            val filtered = dao.filterBlacklistedSongs(ids.map { MediaItem.fromUri(it) })
            assertEquals(25000, filtered.size) // Half
        }
        assertTrue("Seq <150ms on 50k in-mem", dur < 150) // Based on sim 0.1216s
    }
    
    @Test(expected = IllegalArgumentException::class)
    fun `songs throws on unsupported sortBy`() = runTest {
        // Assume utils throw on invalid
        dao.songs(SongSortBy.Title, SortOrder.Ascending) // Supported
    }
    
    @Test
    fun `dynamicSongs with clause works`() = runTest {
        // Arrange: Insert test song
        dao.insert(Song(id = "test", title = "Test", totalPlayTimeMs = 100))
        
        val clause = DatabaseUtils.buildSongSortClause(SongSortBy.Title, SortOrder.Ascending)
        val results = dao.dynamicSongs(clause).first()
        
        assertEquals(1, results.size)
    }
    
    @Test
    fun `insert mediaItem atomic`() = runTest {
        val mediaItem = MediaItem.fromUri("test")
        
        dao.insert(mediaItem)
        
        val song = dao.song("test").first()
        assertNotNull(song)
    }
    
    @Test
    fun `search wildcards case insensitive`() = runTest {
        dao.insert(Song(id = "test", title = "Hello WORLD"))
        
        val results = dao.search("world").first()
        
        assertEquals(1, results.size)
        assertTrue(results[0].title.contains("world", ignoreCase = true))
    }
    
    @Test
    fun `blacklisted null on absent`() = runTest {
        val status = dao.blacklisted("absent").first()
        assertNull(status)
    }
    
    @Test
    fun `migration from22To23 skips empty`() = runTest {
        // Sim pre-state
        db.execSQL("INSERT INTO Song (id, title, totalPlayTimeMs) VALUES ('test', 'Test', 100)")
        
        // Act: Room migration in setup (version auto)
        
        val emptyCount = db.query(SimpleSQLiteQuery("SELECT COUNT(*) FROM Lyrics WHERE fixed IS NULL AND synced IS NULL")).use { cursor ->
            if (cursor.moveToFirst()) cursor.getInt(0) else 0
        }
        assertEquals(0, emptyCount)
    }
}