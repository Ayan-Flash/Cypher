package ai.cypher.client.session.views.tool

import ai.cypher.client.plugin.CypherBundle
import ai.cypher.client.session.model.Tool
import ai.cypher.client.session.ui.selection.SessionSelection
import ai.cypher.client.session.views.SessionViewIcons

/** Renders grep/content-search calls with stacked, clipped search targets. */
class SearchToolView(
    tool: Tool,
    selection: SessionSelection? = null,
    parts: ToolParts = searchParts(3),
    repo: String? = null,
) : BaseSearchToolView(tool, selection, parts, repo) {

    companion object {
        fun canRender(tool: Tool): Boolean = tool.name == "grep"
    }

    override fun toolIcon(tool: Tool) = SessionViewIcons.search
    override fun toolTitle(tool: Tool) = CypherBundle.message("session.part.tool.search")
    override fun targets(tool: Tool, repo: String?) = searchTargets(tool, repo)
    override fun viewName() = "SearchToolView"
}
