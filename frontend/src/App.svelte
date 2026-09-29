<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  let documentData = null;
  let errorMessage = "";

  onMount(async () => {
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

<header><strong>Markdown Reader</strong></header>

<main>
  {#if errorMessage}
    <p class="status error">{errorMessage}</p>
  {:else if documentData}
    <article>{@html documentData.html}</article>
  {:else}
    <p class="status">文書を読み込んでいます…</p>
  {/if}
</main>

<style>
  :global(:root) {
    color-scheme: light;
    font-family: ui-sans-serif, system-ui, sans-serif;
    background: #f3f5f7;
    color: #172033;
  }

  :global(body) { margin: 0; }
  header { padding: 1rem 2rem; background: #172033; color: white; }
  header strong { font-size: 1.05rem; }
  main { width: min(760px, calc(100% - 2rem)); margin: 2rem auto; }
  article {
    padding: clamp(1.5rem, 4vw, 3rem);
    border: 1px solid #dce1e8;
    border-radius: 12px;
    background: white;
    box-shadow: 0 12px 32px rgb(23 32 51 / 8%);
    line-height: 1.75;
  }
  article :global(h1), article :global(h2) { line-height: 1.25; }
  article :global(h1) { padding-bottom: 0.5rem; border-bottom: 2px solid #e6eaf0; }
  article :global(pre) {
    overflow-x: auto;
    padding: 1rem;
    border-radius: 8px;
    background: #172033;
    color: #f7f9fc;
  }
  .status { color: #5c667a; }
  .error { color: #a12626; }
</style>
