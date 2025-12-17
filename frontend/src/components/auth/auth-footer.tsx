import Link from "next/link";

interface AuthFooterProps {
  text: string;
  linkText: string;
  linkHref: string;
}

export function AuthFooter({ text, linkText, linkHref }: AuthFooterProps) {
  return (
    <p className="mt-8 text-center text-sm text-muted-foreground">
      {text}{" "}
      <Link
        href={linkHref}
        className="text-primary hover:underline font-medium transition-colors"
      >
        {linkText}
      </Link>
    </p>
  );
}
