// Mjerilo krivulje napretka: raspon osi Y i razmicanje oznaka uz kraj crta. Bez Reacta.

/** Raspon osi Y u stotinkama: uvijek uključuje 0, zaokružen na 5 bodova (10 kad je raspon velik). */
export function yRange(values: number[]): { lo: number; hi: number; step: number } {
  const lo0 = Math.min(0, ...values);
  const hi0 = Math.max(0, ...values);
  const step = hi0 - lo0 > 4000 ? 1000 : 500;
  let lo = Math.floor(lo0 / step) * step;
  let hi = Math.ceil(hi0 / step) * step;
  if (lo === hi) {
    lo -= step;
    hi += step;
  }
  return { lo, hi, step };
}

/** Oznake uz kraj crta razmaknute najmanje `gap` px, unutar [top, bottom]; ulaz i izlaz redom ulaza. */
export function spreadLabels(ys: number[], gap: number, top: number, bottom: number): number[] {
  const order = ys.map((y, i) => ({ y, i })).sort((a, b) => a.y - b.y);
  const out = order.map((o) => o.y);
  for (let k = 0; k < out.length; k++) {
    const prev = k > 0 ? (out[k - 1] ?? top) + gap : top;
    out[k] = Math.max(out[k] ?? top, prev);
  }
  // Ako je zadnja izašla ispod dna, pomakni sve prema gore.
  for (let k = out.length - 1; k >= 0; k--) {
    const next = k < out.length - 1 ? (out[k + 1] ?? bottom) - gap : bottom;
    out[k] = Math.min(out[k] ?? bottom, next);
  }
  const result = new Array<number>(ys.length);
  order.forEach((o, k) => {
    result[o.i] = out[k] ?? o.y;
  });
  return result;
}
