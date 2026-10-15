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
  <h2 id="language">{t("settings.language")}</h2>
  <p class="help">{t("settings.language.help")}</p>
  <fieldset>
    <legend class="visually-hidden">{t("settings.language")}</legend>
    <div class="languages">
      {#each options as id (id)}
        {@const shown = resolveLanguage(id, systemLanguage)}
        <label class="language">
          <input
            type="radio"
            name="language"
            value={id}
            checked={preferences.language === id}
            onchange={() => setPreference("language", id)}
          />
          <span class="name" lang={id === SYSTEM_LANGUAGE ? undefined : id}>
            {id === SYSTEM_LANGUAGE ? t("settings.language.system") : LANGUAGE_NAMES[shown]}
          </span>
        </label>
      {/each}
    </div>
  </fieldset>
</section>

<style>
  .help {
    margin-bottom: 1.1rem;
    font-size: 0.9rem;
  }
  fieldset {
    margin: 0;
    padding: 0;
    border: 0;
  }
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
    clip-path: inset(50%);
    white-space: nowrap;
  }
  .languages {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(8.5rem, 1fr));
    gap: 0.75rem;
  }
  /* Como las opciones de Tema: una tarjeta por idioma. */
  .language {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
    align-items: center;
    justify-content: center;
    text-align: center;
    padding: 0.65rem 0.5rem 0.5rem;
    border: 1px solid var(--border-strong);
    border-radius: 0.6rem;
    cursor: pointer;
  }
  .language:hover {
    border-color: var(--accent);
  }
  .language:has(input:checked) {
    border-color: var(--accent);
    box-shadow: inset 0 0 0 1px var(--accent);
  }
  .language:has(input:focus-visible) {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
  input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }
  .name {
    padding: 0 0.15rem;
    font-size: 1.05rem;
    font-weight: 600;
  }
  .language:has(input:checked) .name {
    color: var(--accent-text);
  }
</style>
