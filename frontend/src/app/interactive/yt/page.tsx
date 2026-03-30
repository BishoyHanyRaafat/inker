"use client";

import { useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Logo } from "@/components/ui/logo";
import { clearTokens, getAccessToken, isAuthenticated } from "@/lib/api-config";
import { cn } from "@/lib/utils";
import type {
  ProcessedChunk,
  YouTubeStreamResponse,
} from "@/lib/api/types.gen";
import { ArrowLeft, LogOut, Pause, Play, RotateCcw } from "lucide-react";

type QuestionItem =
  | {
      id: string;
      kind: "cloze";
      prompt: string;
      answers: string[];
      fromChunk: number;
    }
  | {
      id: string;
      kind: "flashcard";
      prompt: string;
      answers: string[];
      fromChunk: number;
    };

function extractYouTubeId(input: string): string | null {
  const raw = input.trim();
  if (!raw) return null;

  if (/^[a-zA-Z0-9_-]{11}$/.test(raw)) return raw;
  const watchMatch = raw.match(/[?&]v=([a-zA-Z0-9_-]{11})/);
  if (watchMatch?.[1]) return watchMatch[1];
  const shortMatch = raw.match(/youtu\.be\/([a-zA-Z0-9_-]{11})/);
  if (shortMatch?.[1]) return shortMatch[1];
  const embedMatch = raw.match(/\/embed\/([a-zA-Z0-9_-]{11})/);
  if (embedMatch?.[1]) return embedMatch[1];
  return null;
}

function normalizeYouTubeUrl(input: string): string | null {
  const raw = input.trim();
  if (!raw) return null;
  const id = extractYouTubeId(raw);
  if (id) return `https://www.youtube.com/watch?v=${id}`;
  if (/^https?:\/\//.test(raw)) return raw;
  return null;
}

function toWsBaseUrl(httpBase: string): string {
  if (httpBase.startsWith("https://")) return httpBase.replace("https://", "wss://");
  if (httpBase.startsWith("http://")) return httpBase.replace("http://", "ws://");
  return httpBase;
}

declare global {
  interface Window {
    YT?: any;
    onYouTubeIframeAPIReady?: () => void;
  }
}

export default function InteractiveYouTubeFocusPage() {
  const router = useRouter();

  const [youtubeInput, setYoutubeInput] = useState("");
  const [wsStatus, setWsStatus] = useState<
    "disconnected" | "connecting" | "connected"
  >("disconnected");
  const [lastError, setLastError] = useState<string | null>(null);
  const [chunksSeen, setChunksSeen] = useState(0);

  const [questionQueue, setQuestionQueue] = useState<QuestionItem[]>([]);
  const [activeQuestionIdx, setActiveQuestionIdx] = useState(0);
  const [reveal, setReveal] = useState(false);
  const [attempt, setAttempt] = useState("");

  const wsRef = useRef<WebSocket | null>(null);
  const playerRef = useRef<any>(null);
  const syncTimerRef = useRef<number | null>(null);
  const isPlayingRef = useRef(false);
  const lastTimeSampleRef = useRef<{ t: number; ts: number } | null>(null);
  const bufferingAnchorRef = useRef<number | null>(null);

  const videoId = useMemo(() => extractYouTubeId(youtubeInput), [youtubeInput]);
  const normalizedYoutubeUrl = useMemo(
    () => normalizeYouTubeUrl(youtubeInput),
    [youtubeInput]
  );

  const activeQuestion = questionQueue[activeQuestionIdx] ?? null;

  useEffect(() => {
    if (!isAuthenticated()) {
      router.push("/login");
      return;
    }
  }, [router]);

  useEffect(() => {
    if (!videoId) return;

    const existing = document.querySelector<HTMLScriptElement>(
      'script[data-inker="yt-iframe-api"]'
    );
    if (!existing) {
      const script = document.createElement("script");
      script.src = "https://www.youtube.com/iframe_api";
      script.async = true;
      script.dataset.inker = "yt-iframe-api";
      document.body.appendChild(script);
    }

    const createPlayer = () => {
      if (!window.YT?.Player) return;
      if (playerRef.current) return;

      playerRef.current = new window.YT.Player("yt-player", {
        videoId,
        playerVars: {
          modestbranding: 1,
          rel: 0,
          enablejsapi: 1,
          origin: window.location.origin,
        },
        events: {
          onStateChange: (e: any) => {
            // 1 = playing, 2 = paused, 3 = buffering.
            // We use this for backend state + better seek detection.
            if (e?.data === 1) {
              isPlayingRef.current = true;
              sendEvent({ type: "resumed" });

              // If we were buffering and we land on a different timestamp, it was a seek.
              try {
                const t = playerRef.current?.getCurrentTime?.();
                if (typeof t === "number" && Number.isFinite(t)) {
                  const anchor = bufferingAnchorRef.current;
                  if (typeof anchor === "number" && Math.abs(t - anchor) > 1.5) {
                    sendEvent({ type: "seeked", data: { position: t } });
                  }
                }
              } catch {}

              bufferingAnchorRef.current = null;
            }
            if (e?.data === 2) {
              isPlayingRef.current = false;
              sendEvent({ type: "paused" });
            }
            if (e?.data === 3) {
              // Buffering can be network OR seeking. Capture an anchor time so we can
              // detect a jump when we return to PLAYING.
              isPlayingRef.current = false;
              try {
                const t = playerRef.current?.getCurrentTime?.();
                if (typeof t === "number" && Number.isFinite(t)) {
                  bufferingAnchorRef.current = t;
                }
              } catch {}
            }
          },
        },
      });
    };

    if (window.YT?.Player) {
      createPlayer();
      return;
    }

    const prev = window.onYouTubeIframeAPIReady;
    window.onYouTubeIframeAPIReady = () => {
      prev?.();
      createPlayer();
    };
  }, [videoId]);

  useEffect(() => {
    return () => {
      wsRef.current?.close();
      wsRef.current = null;
      if (syncTimerRef.current) {
        window.clearInterval(syncTimerRef.current);
        syncTimerRef.current = null;
      }
    };
  }, []);

  function wsUrl(): string | null {
    const apiBase = process.env.NEXT_PUBLIC_API_URL || "http://localhost:3000";
    const wsBase = toWsBaseUrl(apiBase);
    // Auth is sent via HttpOnly `access_token` cookie.
    return `${wsBase}/api/v1/interactive/yt/ws`;
  }

  function sendEvent(evt: unknown) {
    if (wsRef.current?.readyState !== WebSocket.OPEN) return;
    wsRef.current.send(JSON.stringify(evt));
  }

  function connectAndStart() {
    setLastError(null);
    setReveal(false);
    setAttempt("");

    const urlForBackend = normalizedYoutubeUrl;
    if (!urlForBackend) {
      setLastError("Please paste a valid YouTube URL (or video id).");
      return;
    }

    const url = wsUrl();
    if (!url) {
      setLastError("Missing WebSocket URL.");
      return;
    }

    if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
      sendEvent({ type: "start", data: { id: urlForBackend } });
      return;
    }

    setWsStatus("connecting");
    // We negotiate a stable protocol name. Auth happens via HttpOnly cookie.
    const ws = new WebSocket(url, ["inker"]);
    wsRef.current = ws;

    ws.onopen = () => {
      setWsStatus("connected");
      sendEvent({ type: "start", data: { id: urlForBackend } });

      try {
        playerRef.current?.playVideo?.();
      } catch {}

      if (!syncTimerRef.current) {
        // Detect real seeks / large jumps (scrubbing, keyboard seeking, jumping while paused).
        // We do NOT send position updates periodically anymore.
        const DRIFT_THRESHOLD_SECS = 1.5;
        syncTimerRef.current = window.setInterval(() => {
          try {
            const t = playerRef.current?.getCurrentTime?.();
            if (typeof t !== "number" || !Number.isFinite(t)) return;

            const now = Date.now();
            const prev = lastTimeSampleRef.current;
            lastTimeSampleRef.current = { t, ts: now };

            // Need a previous sample to compare.
            if (!prev) return;

            let rate = 1;
            try {
              const r = playerRef.current?.getPlaybackRate?.();
              if (typeof r === "number" && Number.isFinite(r) && r > 0) rate = r;
            } catch {}

            // If playing, expect time to advance. If paused/buffering, expect no advance.
            const expectedDelta = isPlayingRef.current
              ? ((now - prev.ts) / 1000) * rate
              : 0;

            const actualDelta = t - prev.t;
            const drift = Math.abs(actualDelta - expectedDelta);

            // Seek = a jump that can't be explained by normal playback progression.
            if (drift > DRIFT_THRESHOLD_SECS) {
              sendEvent({ type: "seeked", data: { position: t } });
            }
          } catch {}
        }, 1000);
      }
    };

    ws.onclose = () => {
      setWsStatus("disconnected");
    };

    ws.onerror = () => {
      setLastError("WebSocket error. Check server logs and your auth token.");
    };

    ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(String(event.data)) as YouTubeStreamResponse;
        if (msg.type === "error") {
          setLastError("Server error while processing the lecture.");
          return;
        }
        if (msg.type !== "chunk") return;

        const chunk: ProcessedChunk = msg.data;
        setChunksSeen((n) => n + 1);

        setQuestionQueue((prev) => {
          const chunkIndex = chunksSeen + 1;
          const next: QuestionItem[] = [];

          for (const c of chunk.cloze_questions || []) {
            next.push({
              id: `c-${chunkIndex}-${prev.length + next.length}`,
              kind: "cloze",
              prompt: c.text,
              answers: c.answers,
              fromChunk: chunkIndex,
            });
          }

          for (const f of chunk.flashcards || []) {
            next.push({
              id: `f-${chunkIndex}-${prev.length + next.length}`,
              kind: "flashcard",
              prompt: f.question,
              answers: [f.answer],
              fromChunk: chunkIndex,
            });
          }

          if (next.length === 0) return prev;
          return [...prev, ...next];
        });
      } catch {
        // ignore malformed messages
      }
    };
  }

  function disconnect() {
    wsRef.current?.close();
    wsRef.current = null;
    setWsStatus("disconnected");
  }

  function nextQuestion() {
    setReveal(false);
    setAttempt("");
    setActiveQuestionIdx((idx) =>
      Math.min(idx + 1, Math.max(0, questionQueue.length - 1))
    );
  }

  function restartQueue() {
    setReveal(false);
    setAttempt("");
    setActiveQuestionIdx(0);
  }

  const canStart = Boolean(normalizedYoutubeUrl);
  const hasQuestions = questionQueue.length > 0;

  return (
    <div className="min-h-screen bg-background bg-gradient-hero">
      <div className="fixed inset-0 bg-grid-pattern opacity-10 pointer-events-none" />

      <header className="sticky top-0 z-50 border-b border-border/50 bg-background/80 backdrop-blur-md">
        <div className="container mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="sm" asChild>
              <Link href="/dashboard" className="flex items-center gap-2">
                <ArrowLeft className="w-4 h-4" />
                Back
              </Link>
            </Button>
            <Logo href="/dashboard" size="sm" />
          </div>

          <div className="flex items-center gap-2">
            <div className="text-xs text-muted-foreground hidden sm:block">
              WS:{" "}
              <span
                className={cn(
                  "font-medium",
                  wsStatus === "connected" && "text-emerald-500",
                  wsStatus === "connecting" && "text-amber-500",
                  wsStatus === "disconnected" && "text-muted-foreground"
                )}
              >
                {wsStatus}
              </span>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                clearTokens();
                router.push("/login");
              }}
              className="text-muted-foreground hover:text-foreground"
            >
              <LogOut className="w-4 h-4 mr-2" />
              Sign Out
            </Button>
          </div>
        </div>
      </header>

      <main className="container mx-auto px-4 py-6 relative z-10">
        <div className="max-w-6xl mx-auto grid grid-cols-1 lg:grid-cols-2 gap-6">
          <Card className="border-border/50 bg-card/50 backdrop-blur-sm">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <span>Focus Lecture</span>
                <div className="text-xs text-muted-foreground">
                  Chunks: <span className="font-medium">{chunksSeen}</span>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="space-y-2">
                <div className="text-sm font-medium">YouTube URL / Video ID</div>
                <Input
                  value={youtubeInput}
                  onChange={(e) => setYoutubeInput(e.target.value)}
                  placeholder="https://www.youtube.com/watch?v=... or dQw4w9WgXcQ"
                  className="bg-card/50 border-border/50"
                />
                {lastError && (
                  <div className="text-sm text-destructive">{lastError}</div>
                )}
              </div>

              <div className="flex flex-wrap gap-3">
                <Button
                  onClick={connectAndStart}
                  disabled={!canStart || wsStatus === "connecting"}
                  className="h-11"
                >
                  <Play className="w-4 h-4 mr-2" />
                  Start (Questions only)
                </Button>
                <Button
                  variant="outline"
                  onClick={() => sendEvent({ type: "paused" })}
                  disabled={wsStatus !== "connected"}
                  className="h-11"
                >
                  <Pause className="w-4 h-4 mr-2" />
                  Pause
                </Button>
                <Button
                  variant="outline"
                  onClick={() => sendEvent({ type: "resumed" })}
                  disabled={wsStatus !== "connected"}
                  className="h-11"
                >
                  <Play className="w-4 h-4 mr-2" />
                  Resume
                </Button>
                <Button
                  variant="outline"
                  onClick={disconnect}
                  disabled={wsStatus === "disconnected"}
                  className="h-11"
                >
                  Disconnect
                </Button>
              </div>

              <div className="rounded-xl overflow-hidden border border-border/50 bg-black/40">
                {videoId ? (
                  <div className="aspect-video w-full">
                    <div id="yt-player" className="w-full h-full" />
                  </div>
                ) : (
                  <div className="aspect-video flex items-center justify-center text-sm text-muted-foreground">
                    Paste a YouTube link to show the player (optional)
                  </div>
                )}
              </div>

              <div className="text-xs text-muted-foreground">
                This mode intentionally shows <span className="font-medium">questions first</span>.
                We’ll use summaries at the end of the lecture.
              </div>
            </CardContent>
          </Card>

          <Card className="border-border/50 bg-card/50 backdrop-blur-sm">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <span>Live Questions</span>
                <div className="flex items-center gap-2">
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={restartQueue}
                    disabled={!hasQuestions}
                    className="text-muted-foreground hover:text-foreground"
                  >
                    <RotateCcw className="w-4 h-4 mr-2" />
                    Restart
                  </Button>
                </div>
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              {!activeQuestion ? (
                <div className="rounded-xl border border-dashed border-border/60 bg-background/40 p-6 text-sm text-muted-foreground">
                  Waiting for the first chunk… once the server generates cloze questions / flashcards,
                  they’ll appear here one-by-one.
                </div>
              ) : (
                <>
                  <div className="text-xs text-muted-foreground">
                    Question {activeQuestionIdx + 1} / {questionQueue.length} •{" "}
                    {activeQuestion.kind.toUpperCase()} • chunk {activeQuestion.fromChunk}
                  </div>

                  <div className="rounded-2xl border border-border/50 bg-background/50 p-5">
                    <div className="text-lg font-semibold leading-snug">
                      {activeQuestion.prompt}
                    </div>
                    <div className="mt-4 space-y-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Your attempt
                      </div>
                      <Textarea
                        value={attempt}
                        onChange={(e) => setAttempt(e.target.value)}
                        placeholder="Type your answer (kept local)"
                        className="min-h-[96px] bg-card/50 border-border/50"
                      />
                    </div>

                    <div className="mt-4 flex flex-wrap gap-3">
                      <Button
                        variant={reveal ? "outline" : "default"}
                        onClick={() => setReveal((v) => !v)}
                        className="h-11"
                      >
                        {reveal ? "Hide answer" : "Reveal answer"}
                      </Button>
                      <Button
                        variant="outline"
                        onClick={nextQuestion}
                        disabled={activeQuestionIdx >= questionQueue.length - 1}
                        className="h-11"
                      >
                        Next
                      </Button>
                    </div>

                    {reveal && (
                      <div className="mt-4 rounded-xl border border-border/50 bg-card/50 p-4">
                        <div className="text-sm font-medium mb-2">Answer</div>
                        <ul className="list-disc pl-5 text-sm text-foreground">
                          {activeQuestion.answers.map((a, i) => (
                            <li key={i}>{a}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </>
              )}

              {questionQueue.length > 0 && (
                <div className="pt-2 border-t border-border/40">
                  <div className="text-xs text-muted-foreground mb-2">
                    Queue (next up)
                  </div>
                  <div className="space-y-2 max-h-40 overflow-auto pr-2">
                    {questionQueue
                      .slice(activeQuestionIdx + 1, activeQuestionIdx + 6)
                      .map((q) => (
                        <div key={q.id} className="text-sm text-muted-foreground truncate">
                          {q.prompt}
                        </div>
                      ))}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        </div>
      </main>
    </div>
  );
}


