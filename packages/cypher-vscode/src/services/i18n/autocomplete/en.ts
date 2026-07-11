// English runtime translations for autocomplete (cypher:autocomplete.* namespace)
// Source: src/i18n/locales/en/cypher.json → "autocomplete" section

export const dict = {
  "cypher:autocomplete.statusBar.enabled": "$(cypher-logo) Autocomplete",
  "cypher:autocomplete.statusBar.snoozed": "snoozed",
  "cypher:autocomplete.statusBar.warning": "$(warning) Autocomplete",
  "cypher:autocomplete.statusBar.tooltip.basic": "Cypher CLI Autocomplete",
  "cypher:autocomplete.statusBar.tooltip.disabled": "Cypher CLI Autocomplete (disabled)",
  "cypher:autocomplete.statusBar.tooltip.noUsableProvider":
    "**No autocomplete model configured**\n\nTo enable autocomplete, add a profile with one of these supported providers: {{providers}}.\n\n[Open Settings]({{command}})",
  "cypher:autocomplete.statusBar.tooltip.sessionTotal": "Session total cost:",
  "cypher:autocomplete.statusBar.tooltip.provider": "Provider:",
  "cypher:autocomplete.statusBar.tooltip.model": "Model:",
  "cypher:autocomplete.statusBar.tooltip.profile": "Profile: ",
  "cypher:autocomplete.statusBar.tooltip.defaultProfile": "Default",
  "cypher:autocomplete.statusBar.tooltip.completionSummary":
    "Performed {{count}} completions between {{startTime}} and {{endTime}}, for a total cost of {{cost}}.",
  "cypher:autocomplete.statusBar.tooltip.providerInfo": "Autocompletions provided by {{model}} via {{provider}}.",
  "cypher:autocomplete.statusBar.cost.zero": "$0.00",
  "cypher:autocomplete.statusBar.cost.lessThanCent": "<$0.01",
  "cypher:autocomplete.toggleMessage": "Cypher CLI Autocomplete {{status}}",
  "cypher:autocomplete.progress.title": "Cypher CLI",
  "cypher:autocomplete.progress.analyzing": "Analyzing your code...",
  "cypher:autocomplete.progress.generating": "Generating suggested edits...",
  "cypher:autocomplete.progress.processing": "Processing suggested edits...",
  "cypher:autocomplete.progress.showing": "Displaying suggested edits...",
  "cypher:autocomplete.input.title": "Cypher CLI: Quick Task",
  "cypher:autocomplete.input.placeholder": "e.g., 'refactor this function to be more efficient'",
  "cypher:autocomplete.commands.generateSuggestions": "Cypher CLI: Generate Suggested Edits",
  "cypher:autocomplete.commands.displaySuggestions": "Display Suggested Edits",
  "cypher:autocomplete.commands.cancelSuggestions": "Cancel Suggested Edits",
  "cypher:autocomplete.commands.applyCurrentSuggestion": "Apply Current Suggested Edit",
  "cypher:autocomplete.commands.applyAllSuggestions": "Apply All Suggested Edits",
  "cypher:autocomplete.commands.category": "Cypher CLI",
  "cypher:autocomplete.codeAction.title": "Cypher CLI: Suggested Edits",
  "cypher:autocomplete.chatParticipant.fullName": "Cypher CLI Agent",
  "cypher:autocomplete.chatParticipant.name": "Agent",
  "cypher:autocomplete.chatParticipant.description": "I can help you with quick tasks and suggested edits.",
  "cypher:autocomplete.incompatibilityExtensionPopup.message":
    "The Cypher CLI Autocomplete is being blocked by a conflict with GitHub Copilot. To fix this, you must disable Copilot's inline suggestions.",
  "cypher:autocomplete.incompatibilityExtensionPopup.disableCopilot": "Disable Copilot",
  "cypher:autocomplete.incompatibilityExtensionPopup.disableInlineAssist": "Disable Autocomplete",
  "cypher:autocomplete.creditsExhausted.message":
    "Cypher CLI Autocomplete has been paused. Possible causes: your Cypher account has no remaining credits, or your configured API key (BYOK) has reached its quota limit. Add Cypher credits or check your API key configuration to resume autocomplete.",
  "cypher:autocomplete.creditsExhausted.addCredits": "Add Credits",
  "cypher:autocomplete.authError.message":
    "Cypher CLI Autocomplete has been paused due to an authentication issue. Possible causes: you are not signed in to Cypher, or your API key (BYOK) is invalid or missing. Please sign in again or check your provider API key settings.",
}
