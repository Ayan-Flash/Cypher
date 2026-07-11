package ai.cypher.client.session.ui.attachment

import ai.cypher.client.app.CypherAppService
import ai.cypher.client.app.CypherSessionService
import ai.cypher.client.session.model.FileAttachment
import ai.cypher.client.testing.FakeAppRpcApi
import ai.cypher.client.testing.FakeSessionRpcApi
import ai.cypher.client.vfs.CypherPath
import ai.cypher.client.vfs.CypherEditorKindRegistry
import ai.cypher.client.vfs.CypherVirtualFile
import ai.cypher.client.vfs.CypherVirtualFileKindRegistry
import ai.cypher.client.vfs.CypherVirtualFileSystem
import ai.cypher.rpc.dto.CypherAppStateDto
import ai.cypher.rpc.dto.CypherAppStatusDto
import ai.cypher.rpc.dto.MessageDto
import ai.cypher.rpc.dto.MessageTimeDto
import ai.cypher.rpc.dto.MessageWithPartsDto
import ai.cypher.rpc.dto.PartDto
import com.intellij.openapi.application.ApplicationManager
import com.intellij.openapi.components.service
import com.intellij.openapi.util.Disposer
import com.intellij.openapi.vfs.VirtualFileManager
import com.intellij.testFramework.fixtures.BasePlatformTestCase
import com.intellij.testFramework.replaceService
import com.intellij.util.ui.UIUtil
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.cancel
import kotlinx.coroutines.runBlocking

class AttachmentEditorKindTest : BasePlatformTestCase() {
    fun testAttachmentParamsUseStableIdentityFields() {
        val item = FileAttachment("part1").apply {
            mime = "text/plain"
            url = "data:text/plain;base64,aGVsbG8="
            filename = "note.txt"
        }

        val params = attachmentParams("ses1", "msg1", item, "note.txt", "/repo")
        val path = CypherPath(AttachmentEditorKind.ID, params).canonical()
        val json = CypherVirtualFileSystem.getInstance().getPath(path)
        val decoded = CypherVirtualFileSystem.decode(json)

        assertEquals(path, decoded)
        assertEquals(AttachmentEditorKind.ID, path.kind)
        assertEquals("ses1", params["sessionId"])
        assertEquals("msg1", params["messageId"])
        assertEquals("part1", params["partId"])
        assertFalse(params["attachmentKey"].isNullOrBlank())
        assertEquals("note.txt", params["filename"])
        assertEquals("text/plain", params["mime"])
        assertEquals("/repo", params["directory"])
        assertFalse(json.contains("projectHash", ignoreCase = true))
        assertFalse(json.contains("launch", ignoreCase = true))
        assertFalse(json.contains("time", ignoreCase = true))
        assertFalse(json.contains("random", ignoreCase = true))
    }

    fun testSameParamsMapToSameVirtualPath() {
        val params = linkedMapOf(
            "sessionId" to "ses1",
            "messageId" to "msg1",
            "partId" to "part1",
            "attachmentKey" to "key1",
            "filename" to "note.txt",
            "mime" to "text/plain",
            "directory" to "/repo",
        )

        val one = CypherVirtualFileSystem.getInstance().getPath(CypherPath(AttachmentEditorKind.ID, params))
        val two = CypherVirtualFileSystem.getInstance().getPath(CypherPath(AttachmentEditorKind.ID, params.toList().reversed().toMap()))

        assertEquals(one, two)
        assertFalse(one.contains("/system/cypher/editors"))
        assertFalse(one.contains("cypherattachment"))
    }

    fun testDuplicatePartAttachmentsMapToDistinctVirtualFiles() {
        val first = FileAttachment("part1").apply {
            mime = "text/plain"
            url = "data:text/plain;base64,b25l"
            filename = "note.txt"
        }
        val second = FileAttachment("part1").apply {
            mime = "text/plain"
            url = "data:text/plain;base64,dHdv"
            filename = "note.txt"
        }
        val one = attachmentParams("ses1", "msg1", first, "note.txt", "/repo")
        val two = attachmentParams("ses1", "msg1", second, "note.txt", "/repo")

        assertFalse(one == two)
        assertFalse(one["attachmentKey"] == two["attachmentKey"])
        assertFalse(CypherPath(AttachmentEditorKind.ID, one) == CypherPath(AttachmentEditorKind.ID, two))
    }

    fun testVirtualFilesAreExcludedFromEditorHistory() {
        ensureAttachmentEditorKind()
        val file = CypherVirtualFile(CypherPath(AttachmentEditorKind.ID, mapOf(
            "directory" to "/repo",
            "sessionId" to "ses1",
            "messageId" to "msg1",
            "partId" to "part1",
            "filename" to "note.txt",
        )))

        assertNull(VirtualFileManager.getInstance().findFileByUrl(file.url))
    }

    fun testAttachmentEditorKindAndVirtualFilesCanBeCleared() {
        ensureAttachmentEditorKind()
        val fs = CypherVirtualFileSystem.getInstance()
        val path = CypherPath(AttachmentEditorKind.ID, mapOf(
            "directory" to "/repo",
            "sessionId" to "ses1",
            "messageId" to "msg1",
            "partId" to "part1",
            "filename" to "note.txt",
        ))
        val file = fs.findOrCreateFile(path)

        assertNotNull(file)
        assertNotNull(service<CypherEditorKindRegistry>().get(AttachmentEditorKind.ID))
        assertNotNull(service<CypherVirtualFileKindRegistry>().get(AttachmentEditorKind.ID))

        unregisterAttachmentEditorKind()
        fs.clear()

        assertNull(service<CypherEditorKindRegistry>().get(AttachmentEditorKind.ID))
        assertNull(service<CypherVirtualFileKindRegistry>().get(AttachmentEditorKind.ID))
        assertNull(fs.findOrCreateFile(path))
    }

    @Suppress("UnstableApiUsage")
    fun testFetchUsesAttachmentKeyBeforeDuplicatePartId() {
        val cs = CoroutineScope(SupervisorJob())
        val app = FakeAppRpcApi()
        val rpc = FakeSessionRpcApi()
        app.state.value = CypherAppStateDto(CypherAppStatusDto.READY)
        val first = PartDto(
            id = "part1",
            sessionID = "ses1",
            messageID = "msg1",
            type = "file",
            mime = "text/plain",
            url = "data:text/plain;base64,b25l",
            filename = "note.txt",
        )
        val second = first.copy(url = "data:text/plain;base64,dHdv")
        rpc.history.add(MessageWithPartsDto(
            info = MessageDto(
                id = "msg1",
                sessionID = "ses1",
                role = "user",
                time = MessageTimeDto(created = 0.0),
            ),
            parts = listOf(first, second),
        ))
        ApplicationManager.getApplication().replaceService(CypherAppService::class.java, CypherAppService(cs, app), testRootDisposable)
        project.replaceService(CypherSessionService::class.java, CypherSessionService(project, cs, rpc), testRootDisposable)
        val item = FileAttachment("part1").apply {
            mime = "text/plain"
            url = second.url.orEmpty()
            filename = "note.txt"
        }
        val results = mutableListOf<AttachmentData>()
        val parent = Disposer.newDisposable()

        try {
            CypherAttachmentEditorService(project, cs).load(ref("ses1", "msg1", item, "note.txt", "/repo"), parent) {
                results.add(it)
            }

            waitFor { results.any { it is AttachmentData.Text } }
            assertTrue(results.any { it is AttachmentData.Connecting })
            val data = results.last { it is AttachmentData.Text } as AttachmentData.Text
            assertEquals("two", data.text)
            assertEquals(1, rpc.attachmentParts.size)
            assertEquals("msg1", rpc.attachmentParts.single().messageId)
            assertEquals(0, rpc.historyCalls)
        } finally {
            Disposer.dispose(parent)
            cs.cancel()
        }
    }

    @Suppress("UnstableApiUsage")
    fun testLoadShowsConnectionFailedUntilRetryBecomesReady() = runBlocking {
        val cs = CoroutineScope(SupervisorJob())
        val app = FakeAppRpcApi()
        val rpc = FakeSessionRpcApi()
        val part = PartDto(
            id = "part1",
            sessionID = "ses1",
            messageID = "msg1",
            type = "file",
            mime = "text/plain",
            url = "data:text/plain;base64,b2s=",
            filename = "note.txt",
        )
        rpc.history.add(MessageWithPartsDto(
            info = MessageDto(
                id = "msg1",
                sessionID = "ses1",
                role = "user",
                time = MessageTimeDto(created = 0.0),
            ),
            parts = listOf(part),
        ))
        app.state.value = CypherAppStateDto(CypherAppStatusDto.ERROR)
        ApplicationManager.getApplication().replaceService(CypherAppService::class.java, CypherAppService(cs, app), testRootDisposable)
        project.replaceService(CypherSessionService::class.java, CypherSessionService(project, cs, rpc), testRootDisposable)
        val item = FileAttachment("part1").apply {
            mime = part.mime.orEmpty()
            url = part.url.orEmpty()
            filename = part.filename.orEmpty()
        }
        val results = mutableListOf<AttachmentData>()
        val parent = Disposer.newDisposable()

        try {
            CypherAttachmentEditorService(project, cs).load(ref("ses1", "msg1", item, "note.txt", "/repo"), parent) {
                results.add(it)
            }

            waitFor { results.any { it is AttachmentData.ConnectionFailed } }
            assertTrue(results.any { it is AttachmentData.Connecting })
            assertTrue(results.any { it is AttachmentData.ConnectionFailed })

            app.state.value = CypherAppStateDto(CypherAppStatusDto.READY)

            waitFor { results.any { it is AttachmentData.Text } }
            val data = results.last { it is AttachmentData.Text } as AttachmentData.Text
            assertEquals("ok", data.text)
        } finally {
            Disposer.dispose(parent)
            cs.cancel()
        }
    }

    private fun waitFor(done: () -> Boolean) {
        val until = System.currentTimeMillis() + 5_000
        while (!done() && System.currentTimeMillis() < until) {
            UIUtil.dispatchAllInvocationEvents()
            Thread.sleep(50)
        }
        assertTrue(done())
    }

    private fun ref(session: String, message: String, item: FileAttachment, name: String, dir: String): AttachmentRef {
        val params = attachmentParams(session, message, item, name, dir)
        return AttachmentRef(
            directory = params.getValue("directory"),
            sessionId = params.getValue("sessionId"),
            messageId = params.getValue("messageId"),
            partId = params.getValue("partId"),
            attachmentKey = params["attachmentKey"],
            filename = params.getValue("filename"),
            mime = params.getValue("mime"),
        )
    }
}
