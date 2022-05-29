<script lang="ts">
  import { SvelteToast } from "@zerodevx/svelte-toast";

  import Editor from "./Editor.svelte";
  import Output from "./Output.svelte";
</script>

<div class="container">
  <!-- <header /> -->
  <div class="editor-container">
    <Editor />
    <Output />
  </div>
</div>
<SvelteToast options={{ dismissable: false }} />

<style>
  .container {
    display: flex;
    flex-direction: column;
    min-height: 100%;
    background-color: #ffffff;
  }

  .editor-container {
    display: flex;
    flex-grow: 1;
    position: relative;
  }

  :global(:root) {
    --toastBackground: #322d3d;
    --toastColor: #dbdee1;
    --toastBorderRadius: 5px;
    --toastBorder: 2px solid rgb(109, 101, 127);
    --toastBarBackground: rgb(255, 101, 35);
    --toastMsgPadding: 0.4em 0.5em 0.8em 0.5em;
    --toastMinHeight: 0;
    --toastWidth: auto;
  }
</style>
