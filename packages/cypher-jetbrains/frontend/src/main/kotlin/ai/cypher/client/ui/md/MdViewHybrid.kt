package ai.cypher.client.ui.md

import ai.cypher.client.session.ui.selection.SessionSelection
import ai.cypher.client.session.ui.style.SessionEditorStyle

internal class MdViewHybrid(
    style: SessionEditorStyle = SessionEditorStyle.current(),
    selection: SessionSelection? = null,
    code: MdCodeBlockFactory = MdCodeBlockFactory.default(),
) : ai.cypher.client.ui.md.hybrid.MdViewHybrid(style, selection, code)
