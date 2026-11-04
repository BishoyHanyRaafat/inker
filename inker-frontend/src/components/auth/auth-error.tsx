import { AlertCircle } from "lucide-react";

interface AuthErrorProps {
  message: string;
}

export function AuthError({ message }: AuthErrorProps) {
  if (!message) return null;

  return (
    <div className="p-3 rounded-lg bg-destructive/10 border border-destructive/20 text-destructive text-sm flex items-center gap-2 animate-fade-in">
      <AlertCircle className="w-4 h-4 shrink-0" />
      {message}
    </div>
  );
}
