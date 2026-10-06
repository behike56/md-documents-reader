<script>
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";
  import ContentTree from "./ContentTree.svelte";
  import contentIndex from "./generated/content-index.json";

  let page = "index";
  let documentData = null;
  let errorMessage = "";
  let loading = true;
  let colorMode = "light";
  let requestId = 0;

  function setColorMode(mode) {
    colorMode = mode;
    document.documentElement.dataset.colorMode = mode;
  }

  function toggleColorMode() {
    setColorMode(colorMode === "dark" ? "light" : "dark");
  }

  async function loadPage() {
    const currentRequest = ++requestId;
    const hash = window.location.hash;
    page = hash.startsWith("#/documents/") ? "document" : "index";
    loading = true;
    errorMessage = "";
    documentData = null;

    try {
      if (page === "document") {
        const path = decodeURIComponent(hash.slice("#/documents/".length));
        const result = await invoke("get_document", { path });
        if (currentRequest !== requestId) return;
        documentData = result;
        document.title = `${result.title} | Markdown Reader`;
      } else {
        document.title = "ファイル構造 | Markdown Reader";
      }
    } catch (error) {
      if (currentRequest !== requestId) return;
      errorMessage = `読み込めませんでした: ${String(error)}`;
    } finally {
      if (currentRequest === requestId) loading = false;
    }
  }

  onMount(() => {
    setColorMode(
      window.matchMedia("(prefers-color-scheme: dark)").matches
        ? "dark"
        : "light",
    );
    window.addEventListener("hashchange", loadPage);
    loadPage();
    return () => window.removeEventListener("hashchange", loadPage);
  });
</script>

<svelte:head>
  <meta
    name="description"
    content="TauriとSvelteで構築したMarkdown文書リーダー"
  />
</svelte:head>

<header>
  <a class="brand" href="#/">Markdown Reader</a>
  <nav aria-label="ページ">
    <a href="#/" aria-current={page === "index" ? "page" : undefined}>ファイル構造</a>
  </nav>
  <button type="button" onclick={toggleColorMode} aria-pressed={colorMode === "dark"}>
    {colorMode === "dark" ? "ライト" : "ダーク"}表示
  </button>
</header>

<main>
  {#if errorMessage}
    <p class="status error">{errorMessage}</p>
  {:else if loading}
    <p class="status">読み込んでいます…</p>
  {:else if page === "index"}
    <section class="index-page" aria-labelledby="index-title">
      <h1 id="index-title">ファイル構造</h1>
      <p class="root-name">backend/content/</p>
      {#if contentIndex.entries.length > 0}
        <ContentTree entries={contentIndex.entries} />
      {:else}
        <p class="status">ファイルはありません。</p>
      {/if}
    </section>
  {:else if documentData}
    <a class="back-link" href="#/">← ファイル構造へ戻る</a>
    <article
      class="markdown-body"
      data-document-theme={documentData.theme}
    >{@html documentData.html}</article>
  {/if}
</main>

<style>
  header {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 0.75rem max(1rem, calc((100% - 760px) / 2));
    background: var(--color-header);
    color: var(--color-header-text);
  }
  .brand { font-size: 1.05rem; font-weight: 700; text-decoration: none; white-space: nowrap; }
  nav { flex: 1; }
  header a { color: inherit; }
  nav a { text-underline-offset: 0.25em; }
  nav a[aria-current="page"] { font-weight: 700; }
  button {
    padding: 0.45rem 0.75rem;
    border: 1px solid var(--color-header-border);
    border-radius: 0.5rem;
    background: transparent;
    color: inherit;
    font: inherit;
    cursor: pointer;
    white-space: nowrap;
  }
  button:hover { background: var(--color-header-hover); }
  button:focus-visible, a:focus-visible { outline: 3px solid var(--color-focus); outline-offset: 2px; }
  main { width: min(760px, calc(100% - 2rem)); margin: 2rem auto; }
  .status { color: var(--color-text-muted); }
  .error { color: var(--color-danger); }
  .index-page { padding: 1.5rem; border: 1px solid var(--color-border); border-radius: 0.75rem; background: var(--color-surface); box-shadow: var(--shadow-document); }
  .index-page h1 { margin-top: 0; }
  .root-name { color: var(--color-text-muted); font-family: ui-monospace, monospace; }
  .back-link { display: inline-block; margin-bottom: 1rem; color: var(--color-text); text-underline-offset: 0.25em; }
  @media (max-width: 600px) {
    header { flex-wrap: wrap; gap: 0.75rem; }
    nav { order: 3; flex-basis: 100%; }
  }
</style>
