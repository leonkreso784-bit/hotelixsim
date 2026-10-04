const nf = new Intl.NumberFormat("hr-HR", { maximumFractionDigits: 2 });

/** Indeks u stotinkama (0–10000) → tekst za prikaz, npr. 5025 → "50,25". */
export function formatIndex(hundredths: number): string {
  return nf.format(hundredths / 100);
}
