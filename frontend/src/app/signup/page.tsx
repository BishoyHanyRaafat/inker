"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { Mail, Lock, User } from "lucide-react";
import {
  AuthLayout,
  OAuthButtons,
  AuthDivider,
  FormInput,
  AuthError,
  SubmitButton,
  AuthFooter,
} from "@/components/auth";
import { signup } from "@/lib/api";
import { setTokens } from "@/lib/api-config";

export default function SignupPage() {
  const router = useRouter();
  const [username, setUsername] = useState("");
  const [email, setEmail] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState("");
  const [isLoading, setIsLoading] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError("");
    setIsLoading(true);

    try {
      const response = await signup({
        body: { username, email, password },
      });

      if (response.data?.data) {
        setTokens(
          response.data.data.access_token.token,
          response.data.data.refresh_token.token
        );
        router.push("/dashboard");
      } else if (response.error) {
        setError(response.error.message || "Signup failed");
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : "Signup failed");
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <AuthLayout
      title="Create an account"
      description="Start organizing your thoughts today"
    >
      <OAuthButtons />

      <AuthDivider />

      <form onSubmit={handleSubmit} className="space-y-5">
        <AuthError message={error} />

        <FormInput
          id="username"
          label="Username"
          type="text"
          value={username}
          onChange={setUsername}
          placeholder="johndoe"
          icon={User}
          required
          disabled={isLoading}
          autoComplete="username"
        />

        <FormInput
          id="email"
          label="Email"
          type="email"
          value={email}
          onChange={setEmail}
          placeholder="you@example.com"
          icon={Mail}
          required
          disabled={isLoading}
          autoComplete="email"
        />

        <FormInput
          id="password"
          label="Password"
          type="password"
          value={password}
          onChange={setPassword}
          placeholder="••••••••"
          icon={Lock}
          required
          disabled={isLoading}
          autoComplete="new-password"
        />

        <SubmitButton isLoading={isLoading} loadingText="Creating account...">
          Create Account
        </SubmitButton>
      </form>

      <AuthFooter
        text="Already have an account?"
        linkText="Sign in"
        linkHref="/login"
      />
    </AuthLayout>
  );
}
