/** Si el error es que no hay conexión con el Servidor: entonces se sale a la pantalla de «Reintentar». */
export function isConnectionError(code: unknown): boolean {
  return code === "connection.lost" || code === "connection.not_connected";
}
