import { useTerminalDimensions } from "@opentui/solid" // cypher_change
import { createEffect, createMemo, createSignal, Show } from "solid-js" // cypher_change
import { useLocal } from "@tui/context/local"
import { useSync } from "@tui/context/sync"
import { map, pipe, flatMap, entries, filter, sortBy, take, groupBy } from "remeda" // cypher_change
import { DialogSelect } from "@tui/ui/dialog-select"
import { useDialog } from "@tui/ui/dialog"
import { createDialogProviderOptions, DialogProvider } from "./dialog-provider"
import { DialogVariant } from "./dialog-variant"
import type { Model } from "@cypher/sdk/v2" // cypher_change
import * as fuzzysort from "fuzzysort"
import { useConnected } from "./use-connected"
import { ModelInfoPanel } from "@/cypher/components/model-info-panel" // cypher_change
import { FreeModelDisclosure } from "@/cypher/components/free-model-disclosure" // cypher_change

export function DialogModel(props: { providerID?: string }) {
  const local = useLocal()
  const sync = useSync()
  const dialog = useDialog()
  const [query, setQuery] = createSignal("")
  const dimensions = useTerminalDimensions() // cypher_change

  const connected = useConnected()
  const providers = createDialogProviderOptions()
  // cypher_change start
  // Memoize anything that iterates all Cypher models to avoid calculating it for
  // each Cypher model and tanking the UI at a couple hundred models
  const cypherRank = createMemo(() => {
    const provider = sync.data.provider.find((provider) => provider.id === "cypher")
    const models = provider?.models ?? {}
    return new Map(Object.entries(models).map(([id, info]) => [id, info.recommendedIndex ?? Infinity] as const))
  })
  // cypher_change end

  const showExtra = createMemo(() => connected() && !props.providerID)

  // cypher_change start
  const wide = createMemo(() => dimensions().width >= 108)
  const [preview, setPreview] = createSignal<{
    model: Model
    provider: string
  }>()

  const lookup = (providerID: string, modelID: string) => {
    const provider = sync.data.provider.find((x) => x.id === providerID)
    const model = provider?.models[modelID]
    if (!provider || !model) return
    return {
      model,
      provider: provider.name,
    }
  }

  createEffect(() => {
    dialog.setSize(wide() ? "xlarge" : "large")
  })

  createEffect(() => {
    const current = local.model.current()
    if (!current) return
    const next = lookup(current.providerID, current.modelID)
    if (!next) return
    setPreview(next)
  })

  const footer = (providerID: string, model: Model) => {
    const labels = [
      providerID === "cypher" && FreeModelDisclosure.hasByok(model) ? FreeModelDisclosure.byok : undefined,
      providerID === "cypher" && FreeModelDisclosure.collectsData(model) ? FreeModelDisclosure.label : undefined,
      model.cost?.input === 0 && providerID === "opencode" ? "Free" : undefined,
    ].filter((label) => label !== undefined)
    return labels.length > 0 ? labels.join(" · ") : undefined
  }
  // cypher_change end

  const options = createMemo(() => {
    const needle = query().trim()
    // cypher_change: removed showSections guard — sections are always built; empty ones are hidden naturally
    const favorites = connected() ? local.model.favorite() : []
    const recents = local.model.recent()

    function toOptions(items: typeof favorites, category: string) {
      if (!showExtra()) return [] // cypher_change
      return items.flatMap((item) => {
        const provider = sync.data.provider.find((x) => x.id === item.providerID)
        if (!provider) return []
        const model = provider.models[item.modelID]
        if (!model) return []
        return [
          {
            key: item,
            value: { providerID: provider.id, modelID: model.id },
            title: model.name ?? item.modelID,
            description: provider.name,
            category,
            disabled: provider.id === "opencode" && model.id.includes("-nano"),
            footer: footer(provider.id, model), // cypher_change
            onSelect: () => {
              onSelect(provider.id, model.id) // cypher_change
            },
          },
        ]
      })
    }

    const favoriteOptions = toOptions(favorites, "Favorites")
    const recentOptions = toOptions(
      recents.filter(
        (item) => !favorites.some((fav) => fav.providerID === item.providerID && fav.modelID === item.modelID),
      ),
      "Recent",
    )

    const providerOptions = pipe(
      sync.data.provider,
      sortBy(
        (provider) => provider.id !== "opencode",
        (provider) => provider.name,
      ),
      flatMap((provider) =>
        pipe(
          provider.models,
          entries(),
          filter(([_, info]) => info.status !== "deprecated"),
          filter(([_, info]) => (props.providerID ? info.providerID === props.providerID : true)),
          map(([model, info]) => ({
            value: { providerID: provider.id, modelID: model },
            title: info.name ?? model,
            releaseDate: info.release_date,
            description: favorites.some((item) => item.providerID === provider.id && item.modelID === model)
              ? "(Favorite)"
              : undefined,
            // cypher_change start
            category: connected()
              ? provider.id === "cypher" && info.recommendedIndex !== undefined
                ? "Recommended"
                : provider.name
              : undefined,
            // cypher_change end
            disabled: provider.id === "opencode" && model.includes("-nano"),
            footer: footer(provider.id, info), // cypher_change
            onSelect() {
              onSelect(provider.id, model) // cypher_change
            },
          })),
          filter((x) => {
            // cypher_change start - only dedupe favorites/recents when those sections are visible
            if (showExtra()) {
              if (favorites.some((item) => item.providerID === x.value.providerID && item.modelID === x.value.modelID))
                return false
              if (recents.some((item) => item.providerID === x.value.providerID && item.modelID === x.value.modelID))
                return false
            }
            // cypher_change end
            return true
          }),
          (options) => sortModelOptions(options, props.providerID !== undefined, cypherRank()), // cypher_change
        ),
      ),
    )

    const popularProviders = !connected()
      ? pipe(
          providers(),
          map((option) => ({
            ...option,
            category: "Popular providers",
          })),
          take(6),
        )
      : []

    // cypher_change start - Filter per-section to preserve group headers while typing
    if (needle) {
      const rank = <U extends { title: string; category?: string }>(items: U[]) =>
        fuzzysort.go(needle, items, { keys: ["title", "category"] }).map((x) => x.obj)
      // rank within each provider category to preserve category order
      const rankedProviders = pipe(
        providerOptions,
        groupBy((x) => x.category ?? ""),
        entries(),
        flatMap(([_, items]) => rank(items)),
      )
      return [...rank(favoriteOptions), ...rank(recentOptions), ...rankedProviders, ...rank(popularProviders)]
    }
    // cypher_change end

    return [...favoriteOptions, ...recentOptions, ...providerOptions, ...popularProviders]
  })

  const provider = createMemo(() =>
    props.providerID ? sync.data.provider.find((x) => x.id === props.providerID) : null,
  )

  const title = createMemo(() => {
    const value = provider()
    if (!value) return "Select model"
    return value.name
  })

  function onSelect(providerID: string, modelID: string) {
    local.model.set({ providerID, modelID }, { recent: true })
    const list = local.model.variant.list()
    const cur = local.model.variant.selected()
    if (cur === "default" || (cur && list.includes(cur))) {
      dialog.clear()
      return
    }
    if (list.length > 0) {
      dialog.replace(() => <DialogVariant />)
      return
    }
    dialog.clear()
  }

  // cypher_change start
  return (
    <box flexDirection="row">
      <box flexGrow={1} flexShrink={1}>
        <DialogSelect<ReturnType<typeof options>[number]["value"]>
          options={options()}
          actions={[
            {
              command: "model.dialog.provider",
              title: connected() ? "Connect provider" : "View all providers",
              onTrigger() {
                dialog.replace(() => <DialogProvider />)
              },
            },
            {
              command: "model.dialog.favorite",
              title: "Favorite",
              disabled: !connected(),
              onTrigger: (option) => {
                local.model.toggleFavorite(option.value as { providerID: string; modelID: string })
              },
            },
          ]}
          onFilter={setQuery}
          onMove={(option) => {
            if (typeof option.value === "string") {
              setPreview(undefined)
              return
            }
            const next = lookup(option.value.providerID, option.value.modelID)
            if (!next) return
            setPreview(next)
          }}
          // cypher_change: removed flat={true} to keep section headers visible while filtering
          skipFilter={true}
          title={title()}
          current={local.model.current()}
        />
      </box>
      <Show when={wide() && preview()}>
        {(item) => <ModelInfoPanel model={item().model} provider={item().provider} />}
      </Show>
    </box>
  )
  // cypher_change end
}

export function sortModelOptions<
  T extends {
    footer?: string
    releaseDate: string
    title: string
    value?: { providerID: string; modelID: string } // cypher_change
  },
>(
  options: T[],
  newestFirst: boolean,
  rank: ReadonlyMap<string, number> = new Map(), // cypher_change
) {
  // cypher_change start - Sort within Recommended / Cypher Gateway
  const recommended = (option: T) =>
    option.value?.providerID === "cypher" ? (rank.get(option.value.modelID) ?? Infinity) : 0
  // cypher_change end
  if (newestFirst)
    return sortBy(
      options,
      recommended, // cypher_change
      [(option) => option.releaseDate, "desc"],
      (option) => option.title,
    )
  return sortBy(
    options,
    recommended, // cypher_change
    (option) => option.footer === undefined, // cypher_change - free model footers include Cypher disclosure labels
    (option) => option.title,
  )
}
