<!-- Barra lateral de las pantallas de trabajo: el nombre de la app y los accesos directos. -->
<script lang="ts">
  import { branding } from "../branding.generated";
  import Places from "./Places.svelte";
</script>

<aside>
  <p class="brand">{branding.name}</p>
  <Places />
</aside>

<style>
  aside {
    display: flex;
    flex-direction: column;
    gap: 1rem;
    padding: 1rem 0.75rem;
    border-right: 1px solid var(--border);
    background: var(--surface);
  }
  .brand {
    margin: 0 0.5rem;
    color: var(--accent-text);
    font-weight: 700;
  }
</style>
