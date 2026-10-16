/**
 * La contraseña de sudo del Administrador (ADR 0007): se pide la primera vez
 * que se cambia una clave y la recuerda el núcleo, solo en memoria. La pinta
 * `components/SudoDialog.svelte`, montado una sola vez.
 */
import { errorMessage } from "../i18n";
import { api } from "../lib/api";

export const sudoPrompt = $state<{ open: boolean; busy: boolean; error?: string }>({ open: false, busy: false });

let pending: ((unlocked: boolean) => void) | null = null;

/** Pide la contraseña hasta que sudo la acepta o se cancela. `wrong`: la recordada ya no valía. */
export function askSudoPassword(wrong = false): Promise<boolean> {
  pending?.(false);
  Object.assign(sudoPrompt, { open: true, busy: false, error: wrong ? errorMessage("admin.sudo_wrong_password") : undefined });
  return new Promise((resolve) => (pending = resolve));
}

/** Prueba `password`; si no vale, el diálogo sigue abierto con el motivo. */
export async function submitSudoPassword(password: string) {
  sudoPrompt.busy = true;
  sudoPrompt.error = undefined;
  try {
    await api.unlockAdmin(password);
    finish(true);
  } catch (code) {
    sudoPrompt.error = errorMessage(code);
  } finally {
    sudoPrompt.busy = false;
  }
}

export function cancelSudoPassword() {
  finish(false);
}

function finish(unlocked: boolean) {
  sudoPrompt.open = false;
  pending?.(unlocked);
  pending = null;
}
