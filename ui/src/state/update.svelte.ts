/** Versión nueva de la app: se busca al arrancar y se ofrece en un aviso. */
import { branding } from "../branding.generated";
import { errorMessage, t } from "../i18n";
import { api } from "../lib/api";
import { isFinished } from "../lib/transferQueue.svelte";
import { confirm } from "./confirm.svelte";
import { toasts } from "./toasts.svelte";
import { transfers } from "./transfers.svelte";

/** Busca una versión nueva y, si la hay, la ofrece. Sin red, no dice nada: ya avisará otro día. */
export async function checkForUpdate() {
  if (!branding.updater) return;
  const available = await api.checkUpdate().catch(() => null);
  if (available) offer(available.version);
}

function offer(version: string) {
  toasts.info(t("update.available", { version }), { label: t("update.install"), run: () => install(version) });
}

async function install(version: string) {
  if (transfers.items.some((transfer) => !isFinished(transfer))) {
    const confirmed = await confirm({
      title: t("update.confirmTitle"),
      body: t("update.confirmHelp"),
      confirmLabel: t("update.install"),
    });
    if (!confirmed) return offer(version);
  }
  const progress = toasts.info(t("update.installing", { version }));
  try {
    // Si sale bien, la app se reinicia ya en la versión nueva.
    await api.installUpdate();
  } catch (code) {
    toasts.dismiss(progress);
    toasts.error(errorMessage(code), { label: t("update.download"), run: () => api.openReleases() });
  }
}
