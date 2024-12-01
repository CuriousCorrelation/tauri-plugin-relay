import { invoke } from '@tauri-apps/api/core'

export type Method =
  | "GET"
  | "POST"
  | "PUT"
  | "DELETE"
  | "PATCH"
  | "HEAD"
  | "OPTIONS"

export type ContentType =
  | { kind: 'text'; content: string }
  | { kind: "json"; content: unknown }
  | { kind: 'form'; content: FormData }
  | {
      kind: 'binary';
      content: Uint8Array;
      mediaType?: string;
      filename?: string;
    }
  | { kind: 'urlencoded'; content: Record<string, string> }

export type AuthType =
  | { kind: "none" }
  | { kind: "basic"; username: string; password: string }
  | { kind: "bearer"; token: string }
  | {
    kind: "digest"
    username: string
    password: string
    realm?: string
    nonce?: string
    opaque?: string
    algorithm?: "MD5" | "SHA-256" | "SHA-512"
    qop?: "auth" | "auth-int"
  }

export type CertificateType =
  | {
    kind: "pem"
    cert: Uint8Array
    key: Uint8Array
  }
  | {
    kind: "pfx"
    data: Uint8Array
    password: string
  }

export interface Request {
  id: number
  url: string
  method: Method
  headers?: Record<string, string[]>
  params?: Record<string, string>
  content?: ContentType
  auth?: AuthType

  security?: {
    certificates?: {
      client?: CertificateType
      ca?: Array<Uint8Array>
    }
    validateCertificates: boolean
    verifyHost: boolean
  }

  proxy?: {
    url: string
  }
}

export interface Response {
  id: number
  status: number
  statusText: string
  headers: Record<string, string[]>
  content: ContentType

  meta: {
    timing: {
      start: number
      end: number
    }
    size: {
      headers: number
      body: number
      total: number
    }
  }
}

export type UnsupportedFeatureError = {
  kind: "unsupported_feature"
  feature: string
  message: string
  interceptor: string
}

export type InterceptorError =
  | UnsupportedFeatureError
  | { kind: "network"; message: string; cause?: unknown }
  | { kind: "timeout"; message: string; phase?: "connect" | "tls" | "response" }
  | { kind: "certificate"; message: string; cause?: unknown }
  | { kind: "parse"; message: string; cause?: unknown }
  | { kind: "abort"; message: string }

export type RequestResult =
  | { kind: 'success'; response: Response }
  | { kind: 'error'; error: InterceptorError }

export async function execute(request: Request): Promise<RequestResult> {
  return await invoke<RequestResult>('plugin:hoppscotch-relay|execute', { request })
}

export async function cancel(requestId: number): Promise<void> {
  return await invoke<void>('plugin:hoppscotch-relay|cancel', { requestId })
}
