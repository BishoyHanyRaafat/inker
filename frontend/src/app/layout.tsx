import type { Metadata, Viewport } from "next";
import { Geist, Geist_Mono } from "next/font/google";
import "./globals.css";

const geistSans = Geist({
  variable: "--font-geist-sans",
  subsets: ["latin"],
});

const geistMono = Geist_Mono({
  variable: "--font-geist-mono",
  subsets: ["latin"],
});

export const metadata: Metadata = {
  title: "Inker - Your Notes, Beautifully Organized",
  description:
    "A modern note-taking app with rich block-based editing. Capture ideas, create lists, and organize your life with lightning speed.",
  keywords: ["notes", "note-taking", "productivity", "organization", "writing"],
  authors: [{ name: "Inker" }],
  openGraph: {
    title: "Inker - Your Notes, Beautifully Organized",
    description:
      "A modern note-taking app with rich block-based editing. Capture ideas, create lists, and organize your life.",
    type: "website",
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: "Inker - Your Notes, Beautifully Organized",
    description:
      "A modern note-taking app with rich block-based editing. Capture ideas, create lists, and organize your life.",
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#ffffff" },
    { media: "(prefers-color-scheme: dark)", color: "#09090b" },
  ],
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en" className="dark" suppressHydrationWarning>
      <body
        className={`${geistSans.variable} ${geistMono.variable} font-sans antialiased min-h-screen bg-background text-foreground`}
      >
        {children}
      </body>
    </html>
  );
}
