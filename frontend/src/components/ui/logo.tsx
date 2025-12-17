import Link from "next/link";
import { PenLine } from "lucide-react";
import { cn } from "@/lib/utils";

interface LogoProps {
  href?: string;
  size?: "sm" | "md" | "lg";
  showText?: boolean;
  className?: string;
}

const sizes = {
  sm: {
    icon: "w-8 h-8",
    iconInner: "w-5 h-5",
    text: "text-xl",
  },
  md: {
    icon: "w-10 h-10",
    iconInner: "w-6 h-6",
    text: "text-2xl",
  },
  lg: {
    icon: "w-12 h-12",
    iconInner: "w-7 h-7",
    text: "text-3xl",
  },
};

export function Logo({
  href = "/",
  size = "md",
  showText = true,
  className,
}: LogoProps) {
  const sizeConfig = sizes[size];

  const content = (
    <div className={cn("flex items-center gap-3", className)}>
      <div
        className={cn(
          "rounded-xl bg-gradient-to-br from-primary to-primary/80 flex items-center justify-center shadow-lg glow-primary transition-all duration-300 group-hover:scale-105",
          sizeConfig.icon
        )}
      >
        <PenLine className={cn("text-primary-foreground", sizeConfig.iconInner)} />
      </div>
      {showText && (
        <span
          className={cn(
            "font-bold tracking-tight bg-gradient-to-r from-foreground to-foreground/70 bg-clip-text text-transparent",
            sizeConfig.text
          )}
        >
          Inker
        </span>
      )}
    </div>
  );

  if (href) {
    return (
      <Link href={href} className="group inline-flex">
        {content}
      </Link>
    );
  }

  return content;
}
