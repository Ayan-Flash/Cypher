import type { Message, Session, Part, SnapshotFileDiff, SessionStatus, Provider } from "@cypher/sdk/v2"
import { createSimpleContext } from "./helper"
import { PreloadMultiFileDiffResult } from "@pierre/diffs/ssr"

export type NormalizedProviderListResponse = {
  all: Map<string, Provider>
  default: {
    [key: string]: string
  }
  connected: Array<string>
}

type Data = {
  agent?: {
    name: string
    color?: string
  }[]
  provider?: NormalizedProviderListResponse
  session: Session[]
  session_status: {
    [sessionID: string]: SessionStatus
  }
  session_diff: {
    [sessionID: string]: SnapshotFileDiff[]
  }
  session_diff_preload?: {
    [sessionID: string]: PreloadMultiFileDiffResult<any>[]
  }
  message: {
    [sessionID: string]: Message[]
  }
  part: {
    [messageID: string]: Part[]
  }
  part_text_accum_delta?: {
    [partID: string]: string
  }
}

export type NavigateToSessionFn = (sessionID: string) => void

export type SessionHrefFn = (sessionID: string) => string

// cypher_change start
export type OpenFileFn = (filePath: string, line?: number, column?: number) => void

export type OpenDiffFn = (diff: {
  file: string
  before?: string // cypher_change - optional, cypher uses `patch`
  after?: string // cypher_change - optional, cypher uses `patch`
  patch?: string // cypher_change
  additions: number
  deletions: number
}) => void

export type OpenUrlFn = (url: string) => void

export type OpenContentFn = (content: string, language?: string) => void // cypher_change

export type ValidateFilesFn = (paths: string[]) => Promise<string[]> // cypher_change
// cypher_change end

export const { use: useData, provider: DataProvider } = createSimpleContext({
  name: "Data",
  init: (props: {
    data: Data
    directory: string
    onNavigateToSession?: NavigateToSessionFn
    onSessionHref?: SessionHrefFn
    onOpenFile?: OpenFileFn // cypher_change
    onOpenDiff?: OpenDiffFn // cypher_change
    onOpenUrl?: OpenUrlFn // cypher_change
    onOpenContent?: OpenContentFn // cypher_change
    onValidateFiles?: ValidateFilesFn // cypher_change
  }) => {
    return {
      get store() {
        return props.data
      },
      get directory() {
        return props.directory
      },
      navigateToSession: props.onNavigateToSession,
      sessionHref: props.onSessionHref,
      openFile: props.onOpenFile, // cypher_change
      openDiff: props.onOpenDiff, // cypher_change
      openUrl: props.onOpenUrl, // cypher_change
      openContent: props.onOpenContent, // cypher_change
      validateFiles: props.onValidateFiles, // cypher_change
    }
  },
})
