@file:Suppress("UnstableApiUsage")

package ai.cypher.backend.rpc

import ai.cypher.backend.app.CypherBackendAppService
import ai.cypher.backend.app.CypherBackendChatManager
import ai.cypher.backend.app.CypherBackendSessionManager
import ai.cypher.backend.workspace.CypherBackendWorkspaceManager
import ai.cypher.log.ChatLogSummary
import ai.cypher.rpc.CypherSessionRpcApi
import ai.cypher.rpc.dto.ChatEventDto
import ai.cypher.rpc.dto.CloudSessionListDto
import ai.cypher.rpc.dto.ConfigUpdateDto
import ai.cypher.rpc.dto.MessageWithPartsDto
import ai.cypher.rpc.dto.ModelSelectionDto
import ai.cypher.rpc.dto.PermissionAlwaysRulesDto
import ai.cypher.rpc.dto.PermissionReplyDto
import ai.cypher.rpc.dto.PermissionRequestDto
import ai.cypher.rpc.dto.PartDto
import ai.cypher.rpc.dto.PromptDto
import ai.cypher.rpc.dto.QuestionReplyDto
import ai.cypher.rpc.dto.QuestionRequestDto
import ai.cypher.rpc.dto.SessionDto
import ai.cypher.rpc.dto.SessionListDto
import ai.cypher.rpc.dto.SessionStatusDto
import com.intellij.openapi.components.service
import ai.cypher.log.CypherLog
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.onCompletion
import kotlinx.coroutines.flow.onStart

/**
 * Backend implementation of [CypherSessionRpcApi].
 *
 * Session CRUD routes through the [CypherBackendWorkspaceManager] to
 * get the correct workspace for a directory. Status tracking and
 * worktree directory management go directly to the
 * [CypherBackendSessionManager]. Chat operations delegate to
 * [CypherBackendChatManager].
 */
class CypherSessionRpcApiImpl internal constructor(
    private val appOverride: CypherBackendAppService? = null,
    private val log: CypherLog = LOG,
    private val source: Flow<ChatEventDto>? = null,
) : CypherSessionRpcApi {
    companion object {
        private val LOG = CypherLog.create(CypherSessionRpcApiImpl::class.java)
    }

    private val workspaces: CypherBackendWorkspaceManager
        get() = app.workspaces

    private val sessions: CypherBackendSessionManager
        get() = app.sessions

    private val chat: CypherBackendChatManager
        get() = app.chat

    private val app: CypherBackendAppService
        get() = appOverride ?: service()

    override suspend fun list(directory: String): SessionListDto =
        ready { workspaces.get(directory).sessions() }

    override suspend fun recent(directory: String, limit: Int): SessionListDto =
        ready { sessions.recent(directory, limit) }

    override suspend fun create(directory: String): SessionDto {
        app.requireReady()
        log.info("create session: directory=$directory")
        return workspaces.get(directory).createSession()
    }

    override suspend fun get(id: String, directory: String): SessionDto {
        app.requireReady()
        val dir = sessions.getDirectory(id, directory)
        return sessions.get(id, dir)
    }

    override suspend fun delete(id: String, directory: String) {
        app.requireReady()
        val dir = sessions.getDirectory(id, directory)
        workspaces.get(dir).deleteSession(id)
    }

    override suspend fun rename(id: String, directory: String, title: String): ai.cypher.rpc.dto.SessionDto {
        app.requireReady()
        val dir = sessions.getDirectory(id, directory)
        return sessions.rename(id, dir, title)
    }

    override suspend fun cloudSessions(directory: String, cursor: String?, limit: Int, gitUrl: String?): CloudSessionListDto =
        ready { sessions.cloudSessions(directory, cursor, limit, gitUrl) }

    override suspend fun importCloudSession(id: String, directory: String): SessionDto =
        ready { sessions.importCloudSession(id, directory) }

    override suspend fun statuses(): Flow<Map<String, SessionStatusDto>> =
        sessions.statuses

    override suspend fun setDirectory(id: String, directory: String) =
        sessions.setDirectory(id, directory)

    override suspend fun getDirectory(id: String, fallback: String): String =
        sessions.getDirectory(id, fallback)

    // ------ chat ------

    override suspend fun enhancePrompt(directory: String, text: String): String =
        ready { chat.enhancePrompt(directory, text) }

    override suspend fun prompt(id: String, directory: String, prompt: PromptDto) {
        app.requireReady()
        log.info("prompt RPC: session=$id, dir=$directory, parts=${prompt.parts.size}")
        chat.prompt(id, directory, prompt)
    }

    override suspend fun command(id: String, directory: String, command: String, arguments: String, prompt: PromptDto) {
        app.requireReady()
        log.info("command RPC: session=$id, dir=$directory, command=$command, parts=${prompt.parts.size}")
        chat.command(id, directory, command, arguments, prompt)
    }

    override suspend fun abort(id: String, directory: String) =
        ready { chat.abort(id, directory) }

    override suspend fun compact(id: String, directory: String, model: ModelSelectionDto) =
        ready { chat.compact(id, directory, model) }

    override suspend fun revert(id: String, directory: String, messageID: String, partID: String?) =
        ready { chat.revert(id, sessions.getDirectory(id, directory), messageID, partID) }

    override suspend fun unrevert(id: String, directory: String) =
        ready { chat.unrevert(id, sessions.getDirectory(id, directory)) }

    override suspend fun messages(id: String, directory: String): List<MessageWithPartsDto> =
        ready { chat.messages(id, directory) }

    override suspend fun attachmentPart(id: String, directory: String, messageId: String, partId: String, attachmentKey: String?): PartDto? =
        ready { chat.attachmentPart(id, directory, messageId, partId, attachmentKey) }

    override suspend fun events(id: String, directory: String): Flow<ChatEventDto> =
        (source ?: chat.events)
            .onStart { log.info("${ChatLogSummary.sid(id)} kind=subscription route=rpc-events start=true dir=${ChatLogSummary.dir(directory)}") }
            .filter { event ->
                val sid = ChatLogSummary.sid(event)
                val passes = event is ChatEventDto.SessionCreated || sid == null || sid == id
                if (passes) log.debug { "${ChatLogSummary.sid(id)} pass=true ${ChatLogSummary.eventBody(event)}" }
                else log.debug { "${ChatLogSummary.sid(id)} pass=false srcSid=$sid ${ChatLogSummary.eventBody(event)}" }
                if (passes) {
                    ChatLogSummary.error(event)?.let { log.warn("${ChatLogSummary.sid(id)} route=rpc-events pass=true $it") }
                }
                if (passes && event is ChatEventDto.SessionStatusChanged && event.status.type != "busy") {
                    log.info(
                        "${ChatLogSummary.sid(id)} kind=status route=rpc-events pass=true " +
                            ChatLogSummary.status(event.status),
                    )
                }
                passes
            }
            .onCompletion { cause ->
                if (cause == null || cause is CancellationException) {
                    log.info("${ChatLogSummary.sid(id)} kind=subscription route=rpc-events stop=true cancelled=${cause is CancellationException}")
                    return@onCompletion
                }
                log.warn("${ChatLogSummary.sid(id)} kind=subscription route=rpc-events stop=true failed message=${cause.message}", cause)
            }

    override suspend fun updateConfig(directory: String, config: ConfigUpdateDto) =
        ready { chat.updateConfig(directory, config) }

    // ------ permission / question resolution ------

    override suspend fun replyPermission(requestId: String, directory: String, reply: PermissionReplyDto) {
        app.requireReady()
        log.info("replyPermission: requestId=$requestId, reply=${reply.reply}")
        chat.replyPermission(requestId, directory, reply)
    }

    override suspend fun savePermissionRules(requestId: String, directory: String, rules: PermissionAlwaysRulesDto) {
        app.requireReady()
        log.info("savePermissionRules: requestId=$requestId")
        chat.savePermissionRules(requestId, directory, rules)
    }

    override suspend fun replyQuestion(requestId: String, directory: String, answers: QuestionReplyDto) {
        app.requireReady()
        log.info("replyQuestion: requestId=$requestId, answers=${answers.answers.size}")
        chat.replyQuestion(requestId, directory, answers)
    }

    override suspend fun rejectQuestion(requestId: String, directory: String) {
        app.requireReady()
        log.info("rejectQuestion: requestId=$requestId")
        chat.rejectQuestion(requestId, directory)
    }

    override suspend fun pendingPermissions(directory: String): List<PermissionRequestDto> =
        ready { chat.pendingPermissions(directory) }

    override suspend fun pendingQuestions(directory: String): List<QuestionRequestDto> =
        ready { chat.pendingQuestions(directory) }

    private suspend fun <T> ready(block: suspend () -> T): T {
        app.requireReady()
        return block()
    }
}
