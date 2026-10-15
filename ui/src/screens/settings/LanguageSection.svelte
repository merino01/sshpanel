<!--
  Ajustes › Idioma: Sistema, Español o English. Cada idioma se llama en su
  propia lengua, para encontrarlo sin entender el activo.
-->
<script lang="ts">
  import { t } from "../../i18n";
  import { systemLanguage } from "../../i18n/locale.svelte";
  import { LANGUAGE_NAMES, LANGUAGES, resolveLanguage, SYSTEM_LANGUAGE } from "../../lib/language";
  import { preferences, setPreference } from "../../state/preferences.svelte";

  const options = [SYSTEM_LANGUAGE, ...LANGUAGES];
</script>

<section aria-labelledby="language">
  <h2 id="language" class="text-xl font-semibold tracking-tight">{t("settings.language")}</h2>
  <p class="mt-1 mb-5 max-w-[60ch] text-muted-foreground">{t("settings.language.help")}</p>
  <fieldset>
    <legend class="sr-only">{t("settings.language")}</legend>
    <!-- Como las opciones de Tema: una tarjeta por idioma. -->
    <div class="grid grid-cols-[repeat(auto-fill,minmax(8.5rem,1fr))] gap-3">
      {#each options as id (id)}
        {@const shown = resolveLanguage(id, systemLanguage)}
        <label class="group relative cursor-pointer rounded-xl border border-input p-2 transition-colors hover:border-primary has-checked:border-primary has-checked:ring-1 has-checked:ring-primary has-focus-visible:outline-2 has-focus-visible:outline-offset-2 has-focus-visible:outline-ring grid place-items-center px-2 py-3 text-center">
          <input
            class="pointer-events-none absolute opacity-0"
            type="radio"
            name="language"
            value={id}
            checked={preferences.language === id}
            onchange={() => setPreference("language", id)}
          />
          <span class="text-base font-semibold group-has-checked:text-accent-text" lang={id === SYSTEM_LANGUAGE ? undefined : id}>
            {id === SYSTEM_LANGUAGE ? t("settings.language.system") : LANGUAGE_NAMES[shown]}
          </span>
        </label>
      {/each}
    </div>
  </fieldset>
</section>
