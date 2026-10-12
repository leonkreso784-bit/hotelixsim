import type { ComponentPropsWithRef, ReactNode } from "react";

// Boja, obrub i unutarnji razmak kartice biraju se propom: iste klase u `className` gube od zadanih (UI_PRAVILA §1.3).
const CARD_TONE = {
  plain: "bg-surface",
  accent: "bg-accent-soft",
  mint: "bg-mint",
  sand: "bg-sand",
} as const;

const CARD_PAD = {
  md: "p-4",
  sm: "p-3",
  /** Popis redaka s razdjelnicima: retci sami nose visinu. */
  list: "px-4 py-1",
  /** Istaknuta brojka (rezultat mjeseca). */
  lg: "px-4 py-6",
} as const;

export function Card({
  children,
  className = "",
  tone = "plain",
  emphasis = false,
  pad = "md",
}: {
  children: ReactNode;
  /** Samo raspored (razmaci, flex, red, vidljivost) — ne boja, obrub ni padding. */
  className?: string;
  tone?: keyof typeof CARD_TONE;
  /** Obrub u boji naglaska (potvrda, izbor). */
  emphasis?: boolean;
  pad?: keyof typeof CARD_PAD;
}) {
  const border = emphasis ? "border-2 border-accent" : "border border-line";
  return (
    <section className={`rounded-card ${border} ${CARD_TONE[tone]} ${CARD_PAD[pad]} ${className}`}>
      {children}
    </section>
  );
}

type ButtonProps = ComponentPropsWithRef<"button"> & {
  variant?: "primary" | "secondary";
  /** "full" (zadano) puni redak; "auto" je širok koliko sadržaj, npr. uz polje unosa. */
  width?: "full" | "auto";
};

export function Button({
  variant = "primary",
  width = "full",
  className = "",
  type = "button",
  ...rest
}: ButtonProps) {
  const look = variant === "primary" ? "bg-accent text-white" : "bg-accent-soft text-accent-ink";
  return (
    <button
      type={type}
      className={`min-h-12 ${width === "full" ? "w-full" : "w-auto shrink-0"} rounded-2xl px-4 py-3 text-base font-bold focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-accent disabled:opacity-50 aria-disabled:cursor-not-allowed aria-disabled:opacity-50 ${look} ${className}`}
      {...rest}
    />
  );
}

const BADGE = {
  accent: "bg-accent-soft text-accent-ink",
  muted: "bg-line text-muted",
  up: "bg-mint text-up",
  down: "bg-sand text-down",
} as const;

export function Badge({ children, tone = "accent" }: { children: ReactNode; tone?: keyof typeof BADGE }) {
  return (
    <span
      className={`inline-flex min-w-7 justify-center rounded-full px-2 py-0.5 text-sm font-bold ${BADGE[tone]}`}
    >
      {children}
    </span>
  );
}

export function StatusText({ children }: { children: ReactNode }) {
  return (
    <p role="status" className="py-2 text-muted">
      {children}
    </p>
  );
}

export function ErrorText({ children }: { children: ReactNode }) {
  return (
    <p role="alert" className="rounded-xl bg-sand px-3 py-2 text-sm text-down">
      {children}
    </p>
  );
}
