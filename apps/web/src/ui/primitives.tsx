import type { ButtonHTMLAttributes, ReactNode } from "react";

export function Card({ children, className = "" }: { children: ReactNode; className?: string }) {
  return (
    <section className={`rounded-card border border-line bg-surface p-4 ${className}`}>{children}</section>
  );
}

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & { variant?: "primary" | "secondary" };

export function Button({ variant = "primary", className = "", type = "button", ...rest }: ButtonProps) {
  const look = variant === "primary" ? "bg-accent text-white" : "bg-accent-soft text-accent";
  return (
    <button
      type={type}
      className={`min-h-12 w-full rounded-2xl px-4 py-3 text-base font-bold disabled:opacity-50 ${look} ${className}`}
      {...rest}
    />
  );
}

const BADGE = {
  accent: "bg-accent-soft text-accent",
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
