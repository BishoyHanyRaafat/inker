import { createContext, useContext, useEffect, useState } from "react"
import githubLight from "highlight.js/styles/github.css?url"
import githubDark from "highlight.js/styles/github-dark.css?url"

type Theme = "dark" | "light" | "system"

type ThemeProviderProps = {
  children: React.ReactNode
  defaultTheme?: Theme
  storageKey?: string
}

type ThemeProviderState = {
  theme: Theme
  setTheme: (theme: Theme) => void
}

const initialState: ThemeProviderState = {
  theme: "system",
  setTheme: () => null,
}

const ThemeProviderContext = createContext<ThemeProviderState>(initialState)

export function ThemeProvider({
  children,
  defaultTheme = "system",
  storageKey = "vite-ui-theme",
  ...props
}: ThemeProviderProps) {
  const [theme, setTheme] = useState<Theme>(
    () => (localStorage.getItem(storageKey) as Theme) || defaultTheme
  )

  useEffect(() => {
    const root = document.documentElement
    const systemTheme = window.matchMedia("(prefers-color-scheme: dark)")

    const applyTheme = () => {
      const resolvedTheme =
        theme === "system"
          ? systemTheme.matches ? "dark" : "light"
          : theme

      root.classList.remove("light", "dark")
      root.classList.add(resolvedTheme)

      let link = document.getElementById(
        "highlight-theme"
      ) as HTMLLinkElement | null

      if (!link) {
        link = document.createElement("link")
        link.id = "highlight-theme"
        link.rel = "stylesheet"
        document.head.appendChild(link)
      }

      link.href = resolvedTheme === "dark" ? githubDark : githubLight
    }

    applyTheme()

    if (theme === "system") {
      systemTheme.addEventListener("change", applyTheme)

      return () => {
        systemTheme.removeEventListener("change", applyTheme)
      }
    }
  }, [theme])

  const value = {
    theme,
    setTheme: (theme: Theme) => {
      localStorage.setItem(storageKey, theme)
      setTheme(theme)
    },
  }

  return (
    <ThemeProviderContext.Provider {...props} value={value}>
      {children}
    </ThemeProviderContext.Provider>
  )
}

export const useTheme = () => {
  const context = useContext(ThemeProviderContext)

  if (context === undefined)
    throw new Error("useTheme must be used within a ThemeProvider")

  return context
}
