<script lang="ts">
  import { Circle2 } from "svelte-loading-spinners";
  import ButtonPanel from "./ButtonPanel.svelte";
  import type { Fail, ServerResponse, Success } from "./vite-env";
  import { loading } from "./loading";
  import { error, response } from "./response";
  import Error from "./Error.svelte";

  function isSuccess(
    response: ServerResponse<Success | Fail>
  ): response is ServerResponse<Success> {
    return response.type == "Success";
  }

  type BorderStatus = "success" | "fail" | "unknown";

  let message = "";
  let metadata = "";
  let status: BorderStatus = "unknown";

  function borderStatusToColor(status: BorderStatus) {
    switch (status) {
      case "success":
        return "#3ad73a";
      case "fail":
        return "#ee5646";
      default:
        return "black";
    }
  }

  $: borderColor = $error != "" ? "yellow" : borderStatusToColor(status);

  $: if ($response != null) {
    if (isSuccess($response)) {
      message = $response.data.output;
      metadata = `Build finished in ${$response.data.elapsed.toFixed(2)}ms`;
      status = "success";
    } else {
      message = "";
      metadata = $response.data as string;
      status = "fail";
    }
  }
</script>

<div class="output-container" style:--bc={borderColor}>
  <ButtonPanel />
  {#if $loading}<div class="spinner-wrapper"><Circle2 /></div>{/if}
  <div class="output">
    {#if $error != ""}
      <Error message={$error} />
    {:else}
      <div class="metadata">{metadata}</div>
      {#if message}<div class="divider" />
        <div class="message">{message}</div>{/if}
    {/if}
  </div>
</div>

<style>
  .spinner-wrapper {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    width: 100%;
    display: grid;
    place-items: center;
  }

  .divider {
    border-bottom: 3px solid white;
  }

  .output-container {
    --bc: black;
    white-space: pre-wrap;
    position: relative;
    padding: 1em;
    background-color: rgb(9, 9, 9);
    font-family: Consolas, monospace;
    flex-basis: 45%;
    border-left: 4px solid var(--bc);
  }

  .output {
    color: white;
  }
</style>
