/// <reference types="svelte" />
/// <reference types="vite/client" />

export type Success = { elapsed: number; output?: string };
export type Fail = string;
export type ResponseType = "Success" | "Error";
export type ServerResponse<Data extends Success | Fail> = { type: ResponseType; data: Data };
