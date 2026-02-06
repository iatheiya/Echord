package app.vitune.providers.github

import java.time.Instant

data class Reactions(
    val url: String,
    val count: Int,
    val likes: Int,
    val dislikes: Int,
    val laughs: Int,
    val confused: Int,
    val hearts: Int,
    val hoorays: Int,
    val eyes: Int,
    val rockets: Int
)

data class SimpleUser(
    val name: String? = null,
    val email: String? = null,
    val login: String,
    val id: Int,
    val nodeId: String,
    val avatarUrl: String,
    val gravatarId: String? = null,
    val url: String,
    val frontendUrl: String,
    val followersUrl: String,
    val followingUrl: String,
    val gistsUrl: String,
    val starredUrl: String,
    val subscriptionsUrl: String,
    val organizationsUrl: String,
    val reposUrl: String,
    val eventsUrl: String,
    val receivedEventsUrl: String,
    val type: String,
    val admin: Boolean
)

data class Release(
    val id: Int,
    val nodeId: String,
    val url: String,
    val frontendUrl: String,
    val assetsUrl: String,
    val tag: String,
    val name: String? = null,
    val markdown: String? = null,
    val draft: Boolean,
    val preRelease: Boolean,
    val createdAt: Instant, // [ИСПРАВЛЕНО] Тип String -> Instant
    val publishedAt: Instant? = null, // [ИСПРАВЛЕНО] Тип String -> Instant
    val author: SimpleUser,
    val assets: List<Asset> = emptyList(),
    val html: String? = null,
    val text: String? = null,
    val discussionUrl: String? = null,
    val reactions: Reactions? = null
) {
    data class Asset(
        val url: String,
        val downloadUrl: String,
        val id: Int,
        val nodeId: String,
        val name: String,
        val label: String? = null,
        val state: State,
        val contentType: String,
        val size: Long,
        val downloads: Int,
        val createdAt: Instant, // [ИСПРАВЛЕНО] Тип String -> Instant
        val updatedAt: Instant, // [ИСПРАВЛЕНО] Тип String -> Instant
        val uploader: SimpleUser? = null
    ) {
        enum class State {
            Uploaded,
            Open
        }
    }
}
