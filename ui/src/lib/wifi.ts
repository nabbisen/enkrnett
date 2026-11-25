// The Wi-Fi QR payload (ENK-UI-025): `WIFI:T:WPA;S:<name>;P:<password>;;`, with the characters
// that the format reserves escaped.

function escapeField(value: string): string {
  return value.replace(/([\\;,:"])/g, '\\$1');
}

export function wifiQrPayload(name: string, password: string): string {
  return `WIFI:T:WPA;S:${escapeField(name)};P:${escapeField(password)};;`;
}

export function addressUrl(address: string): string {
  return `http://${address}/`;
}
