/** Los archivos abiertos con una aplicación del equipo, con sus subidas en la cola de Transferencias de la app. */
import { errorMessage } from "../i18n";
import { api } from "../lib/api";
import { isConnectionError } from "../lib/errors";
import { LocalCopies } from "../lib/localCopies.svelte";
import { choose } from "./confirm.svelte";
import { inWorkspace, showConnectionLost } from "./session.svelte";
import { toasts } from "./toasts.svelte";
import { transfers } from "./transfers.svelte";

export const localCopies = new LocalCopies({
  api,
  queue: transfers,
  choose,
  onError: (code) => {
    if (!isConnectionError(code)) toasts.error(errorMessage(code));
    else if (inWorkspace()) showConnectionLost(code);
  },
});

/** Cada vez que se guarda una copia vigilada, se sube. */
export function watchLocalCopies() {
  void api.onLocalCopyChanged((id) => localCopies.changed(id));
}
