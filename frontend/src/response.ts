import { writable } from "svelte/store";
import type { Fail, ServerResponse, Success } from "./vite-env";

export const response = writable<ServerResponse<Success | Fail>>(null);
export const error = writable<string>("");
