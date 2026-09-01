<script lang="ts">
  import '$lib/styles/global.css';
  import { page } from '$app/stores';
  import { onMount } from 'svelte';
  import { sessionStore } from '$lib/stores/session.svelte';
  import { settingsStore } from '$lib/stores/settings.svelte';
  import Icon from '$lib/components/ui/Icon.svelte';
  import IconButton from '$lib/components/ui/IconButton.svelte';
  import ArgusEye from '$lib/components/eye/ArgusEye.svelte';
  import { goto } from '$app/navigation';

  let { children } = $props();

  // Menubar route uses a different chrome — no sidebar.
  const isMenubar = $derived($page.url.pathname.startsWith('/menubar'));

  onMount(async () => {
    await sessionStore.ensureSubscribed();
    await Promise.all([sessionStore.refresh(), settingsStore.load()]);

    // Auto-navigate to the post-synthesis summary when it completes.
    const { listen } = await import('@tauri-apps/api/event');
    await listen<{ sessionId: string }>('synthesis-complete', (e) => {
      if (!isMenubar) goto(`/summary/${e.payload.sessionId}`);
    });
  });

  const nav = [
    { id: 'sessions',   path: '/',                icon: 'clock-history',  label: 'Session History' },
    { id: 'llm',        path: '/settings/llm/',    icon: 'brain-circuit',  label: 'LLM Config' },
    { id: 'capture',    path: '/settings/capture/',icon: 'aperture',       label: 'Capture Settings' },
    { id: 'vault',      path: '/settings/vault/',  icon: 'book-open',      label: 'Vault Settings' },
  ] as const;

  function isActive(path: string) {
    if (path === '/') return $page.url.pathname === '/' || $page.url.pathname.startsWith('/summary');
    return $page.url.pathname.startsWith(path);
  }
</script>

{#if isMenubar}
  {@render children()}
{:else}
  <div class="frame">
    <aside class="sidebar">
      <div class="brand">
        <ArgusEye state={sessionStore.state.current?.record.status === 'active' ? 'active'
                       : sessionStore.state.current?.record.status === 'paused' ? 'paused'
                       : sessionStore.state.current?.record.status === 'synthesizing' ? 'synthesizing'
                       : 'idle'}
                  size={26} glyph />
      </div>
      <div class="sep" aria-hidden="true"></div>
      <nav>
        {#each nav as item}
          <IconButton
            label={item.label}
            onclick={() => goto(item.path)}
          >
            {#snippet children()}
              <span class="nav-icon" class:active={isActive(item.path)}>
                <Icon name={item.icon as any} size={18} />
              </span>
            {/snippet}
          </IconButton>
        {/each}
      </nav>
      <div class="grow"></div>
    </aside>

    <main class="content" data-tauri-drag-region>
      {@render children()}
    </main>
  </div>
{/if}

<style>
  .frame {
    display: grid;
    grid-template-columns: 48px 1fr;
    height: 100vh;
    background: var(--color-bg-base);
  }

  .sidebar {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 10px 0;
    border-right: 1px solid var(--color-border-subtle);
    background: var(--color-bg-base);
  }
  .brand {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px; height: 32px;
    margin-bottom: 4px;
  }
  .sep {
    width: 24px;
    height: 1px;
    background: var(--color-border-subtle);
    margin: 6px 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .nav-icon {
    display: inline-flex;
    color: var(--color-text-tertiary);
    transition: color var(--duration-fast) var(--ease-default);
  }
  .nav-icon.active { color: var(--color-brass); }
  .grow { flex: 1; }

  .content {
    overflow-y: auto;
    background: var(--color-bg-base);
    position: relative;
  }
  .content::before {
    content: '';
    position: absolute;
    inset: 0;
    pointer-events: none;
    background-image: url("data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='80' height='80'><filter id='n'><feTurbulence baseFrequency='0.9' numOctaves='2' /></filter><rect width='100%' height='100%' filter='url(%23n)' opacity='0.006'/></svg>");
    background-size: 80px 80px;
  }
</style>
