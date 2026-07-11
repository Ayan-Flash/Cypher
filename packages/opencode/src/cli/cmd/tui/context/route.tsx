import { createStore, reconcile, unwrap } from "solid-js/store" // cypher_change
import { createSimpleContext } from "./helper"
import type { PromptInfo } from "../component/prompt/history"

export type HomeRoute = {
  type: "home"
  prompt?: PromptInfo
}

export type SessionRoute = {
  type: "session"
  sessionID: string
  prompt?: PromptInfo
}

// cypher_change start
export type CypherClawRoute = {
  type: "cypherclaw"
}
// cypher_change end

export type PluginRoute = {
  type: "plugin"
  id: string
  data?: Record<string, unknown>
}

export type Route = HomeRoute | SessionRoute | PluginRoute | CypherClawRoute // cypher_change

export const { use: useRoute, provider: RouteProvider } = createSimpleContext({
  name: "Route",
  init: (props: { initialRoute?: Route }) => {
    const [store, setStore] = createStore<Route>(
      props.initialRoute ??
        (process.env["CYPHER_ROUTE"]
          ? JSON.parse(process.env["CYPHER_ROUTE"])
          : {
              type: "home",
            }),
    )

    // cypher_change start
    let previous: Route | undefined
    // cypher_change end

    return {
      get data() {
        return store
      },
      navigate(route: Route) {
        previous = structuredClone(unwrap(store)) // cypher_change
        setStore(reconcile(route))
      },
      // cypher_change start
      back() {
        const target = previous ?? ({ type: "home" } as const)
        previous = undefined
        console.log("navigate", target)
        setStore(target)
      },
      // cypher_change end
    }
  },
})

export type RouteContext = ReturnType<typeof useRoute>

export function useRouteData<T extends Route["type"]>(type: T) {
  const route = useRoute()
  return route.data as Extract<Route, { type: typeof type }>
}
