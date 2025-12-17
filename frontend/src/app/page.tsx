import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Logo } from "@/components/ui/logo";
import {
  LayoutGrid,
  Lock,
  Zap,
  ArrowRight,
  Sparkles,
  FileText,
  FolderKanban,
} from "lucide-react";

export default function Home() {
  const currentYear = new Date().getFullYear();

  return (
    <div className="min-h-screen flex flex-col bg-background bg-gradient-hero relative overflow-hidden">
      {/* Background decorations */}
      <div className="absolute inset-0 bg-grid-pattern opacity-30" />
      <div className="absolute top-1/4 -left-1/4 w-1/2 h-1/2 bg-primary/5 rounded-full blur-3xl" />
      <div className="absolute bottom-1/4 -right-1/4 w-1/2 h-1/2 bg-primary/5 rounded-full blur-3xl" />

      {/* Navigation */}
      <nav className="relative z-10 border-b border-border/50 backdrop-blur-sm">
        <div className="max-w-6xl mx-auto px-6 py-4 flex items-center justify-between">
          <Logo href="/" size="sm" />
          <div className="flex items-center gap-3">
            <Button variant="ghost" asChild className="hidden sm:inline-flex">
              <Link href="/login">Sign In</Link>
            </Button>
            <Button asChild className="group">
              <Link href="/signup" className="flex items-center gap-2">
                Get Started
                <ArrowRight className="w-4 h-4 transition-transform group-hover:translate-x-1" />
              </Link>
            </Button>
          </div>
        </div>
      </nav>

      {/* Hero Section */}
      <main className="relative z-10 flex-1 flex flex-col items-center justify-center px-6 py-16 md:py-24">
        <div className="max-w-4xl mx-auto text-center">
          {/* Badge */}
          <div className="inline-flex items-center gap-2 px-4 py-2 rounded-full bg-primary/10 border border-primary/20 mb-8 animate-fade-in-up">
            <Sparkles className="w-4 h-4 text-primary" />
            <span className="text-sm font-medium text-primary">
              Now with AI-powered features
            </span>
          </div>

          {/* Headline */}
          <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-bold tracking-tight mb-6 animate-fade-in-up delay-100">
            <span className="gradient-text">Your thoughts,</span>
            <br />
            <span className="text-foreground">beautifully organized</span>
          </h1>

          {/* Subheadline */}
          <p className="text-lg md:text-xl text-muted-foreground mb-10 max-w-2xl mx-auto leading-relaxed animate-fade-in-up delay-200">
            A modern note-taking app with rich block-based editing. Capture
            ideas, create lists, and organize your life with lightning speed.
          </p>

          {/* CTA Buttons */}
          <div className="flex flex-col sm:flex-row gap-4 justify-center animate-fade-in-up delay-300">
            <Button
              asChild
              size="lg"
              className="text-base px-8 h-12 shadow-lg glow-primary-hover transition-all duration-300"
            >
              <Link href="/signup" className="flex items-center gap-2">
                Start for Free
                <ArrowRight className="w-4 h-4" />
              </Link>
            </Button>
            <Button
              asChild
              variant="outline"
              size="lg"
              className="text-base px-8 h-12 backdrop-blur-sm"
            >
              <Link href="/login">Sign In to Your Account</Link>
            </Button>
          </div>

          {/* Trust indicators */}
          <div className="mt-12 flex flex-wrap items-center justify-center gap-6 text-sm text-muted-foreground animate-fade-in-up delay-400">
            <div className="flex items-center gap-2">
              <Lock className="w-4 h-4" />
              <span>End-to-end encrypted</span>
            </div>
            <div className="hidden sm:block w-1 h-1 rounded-full bg-muted-foreground/50" />
            <div className="flex items-center gap-2">
              <Zap className="w-4 h-4" />
              <span>Blazing fast with Rust</span>
            </div>
            <div className="hidden sm:block w-1 h-1 rounded-full bg-muted-foreground/50" />
            <div className="flex items-center gap-2">
              <FileText className="w-4 h-4" />
              <span>Free forever</span>
            </div>
          </div>
        </div>

        {/* Feature Grid */}
        <div className="mt-20 md:mt-28 grid grid-cols-1 md:grid-cols-3 gap-6 max-w-5xl mx-auto px-6 w-full">
          <FeatureCard
            icon={<LayoutGrid className="w-6 h-6" />}
            title="Block-based editing"
            description="Build your notes with flexible blocks - text, images, todos, tables, and more."
            delay="delay-100"
          />
          <FeatureCard
            icon={<Lock className="w-6 h-6" />}
            title="Secure & Private"
            description="Your notes are encrypted and protected. Sign in with Google, GitHub, or email."
            delay="delay-200"
          />
          <FeatureCard
            icon={<Zap className="w-6 h-6" />}
            title="Lightning fast"
            description="Built with Rust on the backend for blazing performance. Your notes load instantly."
            delay="delay-300"
          />
        </div>

        {/* Secondary features */}
        <div className="mt-8 grid grid-cols-1 md:grid-cols-2 gap-6 max-w-3xl mx-auto px-6 w-full">
          <Card className="group card-hover border-border/50 bg-card/50 backdrop-blur-sm animate-fade-in-up delay-400">
            <CardContent className="pt-6 flex items-start gap-4">
              <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary shrink-0">
                <FolderKanban className="w-5 h-5" />
              </div>
              <div>
                <h3 className="font-semibold mb-1">Smart Organization</h3>
                <p className="text-sm text-muted-foreground">
                  Organize with folders, tags, and powerful search
                </p>
              </div>
            </CardContent>
          </Card>
          <Card className="group card-hover border-border/50 bg-card/50 backdrop-blur-sm animate-fade-in-up delay-500">
            <CardContent className="pt-6 flex items-start gap-4">
              <div className="w-10 h-10 rounded-lg bg-primary/10 flex items-center justify-center text-primary shrink-0">
                <Sparkles className="w-5 h-5" />
              </div>
              <div>
                <h3 className="font-semibold mb-1">AI Assistant</h3>
                <p className="text-sm text-muted-foreground">
                  Get help writing, summarizing, and brainstorming
                </p>
              </div>
            </CardContent>
          </Card>
        </div>
      </main>

      {/* Footer */}
      <footer className="relative z-10 border-t border-border/50 py-8 px-6 backdrop-blur-sm">
        <div className="max-w-5xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4 text-sm text-muted-foreground">
          <p>© {currentYear} Inker. All rights reserved.</p>
          <div className="flex gap-6">
            <Link
              href="#"
              className="hover:text-foreground transition-colors duration-200"
            >
              Privacy
            </Link>
            <Link
              href="#"
              className="hover:text-foreground transition-colors duration-200"
            >
              Terms
            </Link>
            <a
              href="https://github.com"
              target="_blank"
              rel="noopener noreferrer"
              className="hover:text-foreground transition-colors duration-200"
            >
              GitHub
            </a>
          </div>
        </div>
      </footer>
    </div>
  );
}

function FeatureCard({
  icon,
  title,
  description,
  delay,
}: {
  icon: React.ReactNode;
  title: string;
  description: string;
  delay?: string;
}) {
  return (
    <Card
      className={`group card-hover border-border/50 bg-card/50 backdrop-blur-sm animate-fade-in-up ${delay}`}
    >
      <CardContent className="pt-6">
        <div className="w-12 h-12 rounded-xl bg-primary/10 flex items-center justify-center text-primary mb-4 transition-all duration-300 group-hover:bg-primary group-hover:text-primary-foreground group-hover:scale-110">
          {icon}
        </div>
        <h3 className="text-lg font-semibold mb-2">{title}</h3>
        <p className="text-muted-foreground text-sm leading-relaxed">
          {description}
        </p>
      </CardContent>
    </Card>
  );
}
