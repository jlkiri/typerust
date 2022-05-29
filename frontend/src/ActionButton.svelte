<script>
  export let disabled = false;
  export let dark = true;
</script>

<button {disabled} class:dark on:click><slot /></button>

<style>
  button {
    padding: 0.4em 0.8em;
    border-radius: 5px;
    font-family: "Segoe UI", Tahoma, Geneva, Verdana, sans-serif;
    background: #ffffff;
    border: 2px solid rgb(51, 47, 59);
    color: black;
    font-size: 1.1rem;
    transition: 200ms;
    filter: brightness(0.9);
    font-weight: bold;
  }

  button.dark {
    background: #322d3d;
    border: 2px solid rgb(109, 101, 127);
    color: white;
  }

  button:disabled {
    filter: grayscale() brightness(0.6);
  }

  button:hover {
    cursor: pointer;
    filter: brightness(1.1);
  }
</style>
