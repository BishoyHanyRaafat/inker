import { Separator } from "@/components/ui/separator";

interface AuthDividerProps {
  text?: string;
}

export function AuthDivider({ text = "Or continue with email" }: AuthDividerProps) {
  return (
    <div className="relative my-10">
      <div className="absolute inset-0 flex items-center">
        <Separator className="w-full" />
      </div>
      <div className="relative flex justify-center text-xs uppercase">
        <span className="bg-card px-3 text-muted-foreground">{text}</span>
      </div>
    </div>
  );
}
