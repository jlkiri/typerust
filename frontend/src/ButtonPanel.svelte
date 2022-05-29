<script lang="ts">
  import ActionButton from "./ActionButton.svelte";
  import { code } from "./code";
  import { BUILD_URL, RUN_URL } from "./const";
  import { loading } from "./loading";
  import { error, response } from "./response";
  import lz from "lz-string";
  import { toast } from "@zerodevx/svelte-toast";

  async function request(url: string) {
    $loading = true;
    try {
      const resp = await fetch(url, {
        method: "POST",
        body: $code,
      });

      if (resp.status >= 400) {
        $error = `The server responded with a ${resp.status} error: ${resp.statusText}. This is likely an internal playground problem.`;
        return;
      }

      const json = await resp.json();
      $response = json;
      $error = "";
    } catch (e) {
      $error = `Failed to send a network request. Check your network connection and try again.`;
    } finally {
      $loading = false;
    }
  }

  function build() {
    request(BUILD_URL);
  }

  function execute() {
    request(RUN_URL);
  }

  async function copyPageUrl() {
    try {
      await navigator.clipboard.writeText(location.href);
      console.log("Page URL copied to clipboard");
    } catch (err) {
      console.error("Failed to copy: ", err);
    }
  }

  function share() {
    const codeText = $code;
    const compressedCode = lz.compressToEncodedURIComponent(codeText);
    location.hash = compressedCode;
    copyPageUrl();
    toast.push("Copied URL to clipboard", { duration: 2000 });
  }
</script>

<div class="panel">
  <ActionButton disabled={$loading} on:click={build}
    >Build <span>🛠️</span></ActionButton
  >
  <ActionButton disabled={$loading} on:click={execute}
    >Run <span>▶️</span></ActionButton
  >
  <ActionButton on:click={share}>Share 🪄</ActionButton>
  <a href="/about">About TypeRust</a>
</div>

<style>
  .panel a {
    display: block;
    margin-left: auto;
    color: salmon;
    align-self: top;
  }

  .panel {
    margin-bottom: 1em;
    display: flex;
    width: 100%;
    gap: 0.5em;
  }
</style>
