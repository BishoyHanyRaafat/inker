"use client";

import { Button } from "@/components/ui/button";
import { Loader2 } from "lucide-react";

interface SubmitButtonProps {
  isLoading: boolean;
  loadingText: string;
  children: React.ReactNode;
}

export function SubmitButton({ isLoading, loadingText, children }: SubmitButtonProps) {
  return (
    <Button
      type="submit"
      className="w-full h-11 text-base font-medium transition-all duration-200 mt-2"
      disabled={isLoading}
    >
      {isLoading ? (
        <>
          <Loader2 className="w-4 h-4 animate-spin" />
          {loadingText}
        </>
      ) : (
        children
      )}
    </Button>
  );
}
