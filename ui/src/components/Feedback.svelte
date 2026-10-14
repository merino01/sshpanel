<!-- Qué ha hecho la última operación o, si ha fallado, por qué. El error tiene prioridad. -->
<script lang="ts">
  let { notice, message }: { notice?: string; message?: string } = $props();
</script>

{#if message}
  <p class="error" role="alert">{message}</p>
{:else if notice}
  <p class="feedback" role="status">{notice}</p>
{/if}
