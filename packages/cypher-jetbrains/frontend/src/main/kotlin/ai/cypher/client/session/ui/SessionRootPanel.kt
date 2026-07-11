package ai.cypher.client.session.ui

import ai.cypher.client.ui.LayeredOverlayPanel

class SessionRootPanel(
    private val sessionOverlay: Overlay = Overlay(),
    private val sessionBlocker: Blocker = Blocker(),
) : LayeredOverlayPanel(overlay = sessionOverlay, blocker = sessionBlocker) {
    override val overlay: Overlay get() = sessionOverlay

    override val blocker: Blocker get() = sessionBlocker

    class Overlay : LayeredOverlayPanel.Overlay()

    class Blocker : LayeredOverlayPanel.Blocker()
}
