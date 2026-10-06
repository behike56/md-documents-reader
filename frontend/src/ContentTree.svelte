<script>
  import ContentTree from "./ContentTree.svelte";

  let { entries } = $props();
</script>

<ul>
  {#each entries as entry (entry.path)}
    <li>
      {#if entry.kind === "directory"}
        <span class="directory">📁 {entry.name}/</span>
        {#if entry.children.length > 0}
          <ContentTree entries={entry.children} />
        {/if}
      {:else if entry.kind === "markdown"}
        <a href={`#/documents/${encodeURIComponent(entry.path)}`}>📄 {entry.page}. {entry.title}</a>
        <span class="file-name">{entry.name}</span>
        {#if entry.description}
          <p class="description">{entry.description}</p>
        {/if}
        <ul class="metadata" aria-label="大・中・小カテゴリーとタグ">
          <li class="category">大: {entry.categories.large}</li>
          <li class="category">中: {entry.categories.medium}</li>
          <li class="category">小: {entry.categories.small}</li>
          {#each entry.tags || [] as tag}
            <li class="tag">#{tag}</li>
          {/each}
        </ul>
      {/if}
    </li>
  {/each}
</ul>

<style>
  ul { list-style: none; margin: 0; padding-left: 1.5rem; }
  li { margin: 0.4rem 0; }
  .directory { font-weight: 650; }
  .file-name { margin-left: 0.5rem; color: var(--color-text-muted); font-size: 0.875em; }
  .description { margin: 0.25rem 0 0.4rem 1.5rem; color: var(--color-text-muted); }
  .metadata { display: flex; flex-wrap: wrap; gap: 0.3rem; margin: 0.3rem 0 0.7rem 1.5rem; padding: 0; }
  .metadata li { margin: 0; padding: 0.15rem 0.5rem; border-radius: 1rem; background: var(--color-surface-muted); font-size: 0.8rem; }
  .tag { color: var(--color-text-muted); }
  a { color: var(--color-text); text-underline-offset: 0.2em; }
  a:hover { color: var(--color-text-muted); }
  a:focus-visible { outline: 3px solid var(--color-focus); outline-offset: 2px; }
</style>
