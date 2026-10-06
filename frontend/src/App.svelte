<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let documentData = null;
  let errorMessage = "";
  let colorMode = "light";

  function setColorMode(mode) {
    colorMode = mode;
    document.documentElement.dataset.colorMode = mode;
  }

  function toggleColorMode() {
    setColorMode(colorMode === "dark" ? "light" : "dark");
  }

  onMount(async () => {
    setColorMode(
      window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light",
    );

    try {
      documentData = await invoke("get_document");
      document.title = `${documentData.title} | Markdown Reader`;
    } catch (error) {
      errorMessage = `文書を読み込めませんでした: ${String(error)}`;
    }
  });
</script>

<svelte:head>
  <meta
    name="description"
    content="TauriとSvelteで構築したMarkdown文書リーダー"
  />
</svelte:head>

<header>
  <strong>Markdown Reader</strong>
  <button type="button" onclick={toggleColorMode} aria-pressed={colorMode === "dark"}>
    {colorMode === "dark" ? "ライト" : "ダーク"}表示
  </button>
</header>

<main>
  {#if errorMessage}
    <p class="status error">{errorMessage}</p>
  {:else if documentData}
    <article
      class="markdown-body"
      data-document-theme={documentData.theme}
    >{@html documentData.html}</article>
  {:else}
    <p class="status">文書を読み込んでいます…</p>
  {/if}
</main>

<style>
  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    padding: 0.75rem max(1rem, calc((100% - 760px) / 2));
    background: var(--color-header);
    color: var(--color-header-text);
  }
  header strong { font-size: 1.05rem; }
  button {
    padding: 0.45rem 0.75rem;
    border: 1px solid var(--color-header-border);
    border-radius: 0.5rem;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }
  button:hover { background: var(--color-header-hover); }
  button:focus-visible { outline: 3px solid var(--color-focus); outline-offset: 2px; }
  main { width: min(760px, calc(100% - 2rem)); margin: 2rem auto; }
  .status { color: var(--color-text-muted); }
  .error { color: var(--color-danger); }
</style>
