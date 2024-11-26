import { invoke } from '@tauri-apps/api/core'

export interface KeyValuePair {
  key: string
  value: string
}

export type FormDataValue =
  | { type: 'Text'; value: string }
  | {
    type: 'File'
    filename: string
    data: Uint8Array
    mime: string
  }

export interface FormDataEntry {
  key: string
  value: FormDataValue
}

export type BodyDef =
  | { type: 'Text'; content: string }
  | { type: 'URLEncoded'; content: KeyValuePair[] }
  | { type: 'FormData'; content: FormDataEntry[] }

export type ClientCertDef =
  | {
    type: 'PEMCert'
    certificatePem: Uint8Array
    keyPem: Uint8Array
  }
  | {
    type: 'PFXCert'
    certificatePfx: Uint8Array
    password: string
  }

export interface ProxyConfig {
  url: string
}

export interface RequestWithMetadata {
  reqId: number
  method: string
  endpoint: string
  headers: KeyValuePair[]
  body?: BodyDef
  validateCerts: boolean
  rootCertBundleFiles: Uint8Array[]
  clientCert?: ClientCertDef
  proxy?: ProxyConfig
}

export interface ResponseWithMetadata {
  status: number
  statusText: string
  headers: KeyValuePair[]
  data: Uint8Array
  timeStartMs: number
  timeEndMs: number
}

export enum RelayError {
  InvalidMethod = 'InvalidMethod',
  InvalidUrl = 'InvalidUrl',
  InvalidHeaders = 'InvalidHeaders',
  RequestCancelled = 'RequestCancelled',
  RequestRunError = 'RequestRunError'
}

export type RelayResult<T> = T | RelayError

export interface RunOptions {
  req: RequestWithMetadata
}

export interface RunResponse {
  value: RelayResult<ResponseWithMetadata>
}

export interface CancelOptions {
  reqId: number
}

export interface CancelResponse {}

export async function run(options: RunOptions): Promise<RunResponse> {
  return await invoke<RunResponse>('plugin:hoppscotch-relay|run', { options })
}

export async function cancel(options: CancelOptions): Promise<CancelResponse> {
  return await invoke<CancelResponse>('plugin:hoppscotch-relay|cancel', { options })
}
