import { useEffect, useMemo, useRef, useState } from "react";
import { Link, useNavigate } from "react-router-dom";

import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Textarea } from "@/components/ui/textarea";
import { Logo } from "@/components/ui/logo";

import { clearTokens, isAuthenticated } from "../lib/api-config";
import { cn } from "../lib/utils";

import type {
  ClozeQuestion,
  FlashCardQuestion,
  OpenEndedQuestion,
} from "../lib/api/types.gen";

import { ArrowLeft, LogOut, Play, RotateCcw } from "lucide-react";
import { ytQuestionHandler, ytSummaryHandler } from "@/lib/api";

type SectionQuestions = {
  cloze: ClozeQuestion[];
  flashcards: FlashCardQuestion[];
  open_ended: OpenEndedQuestion[];
};

type ScheduledQuestion = {
  trigger_at: number;
  questions: SectionQuestions;
};

type Question =
  | { type: "cloze"; data: ClozeQuestion; id: string }
  | { type: "flashcard"; data: FlashCardQuestion; id: string }
  | { type: "open_ended"; data: OpenEndedQuestion; id: string };

type PlaybackStatus = "idle" | "generating" | "playing" | "waiting";

declare global {
  interface Window {
    YT?: {
      Player: new (
        element: string | HTMLElement,
        options: {
          videoId: string;
          playerVars?: Record<string, unknown>;
          events?: {
            onReady?: (event: any) => void;
            onStateChange?: (event: any) => void;
            onError?: (event: any) => void;
          };
        }
      ) => any;
    };
    onYouTubeIframeAPIReady?: () => void;
  }
}

function extractYouTubeId(input: string): string | null {
  const raw = input.trim();

  if (!raw) return null;

  if (/^[a-zA-Z0-9_-]{11}$/.test(raw)) {
    return raw;
  }

  const watchMatch = raw.match(/[?&]v=([a-zA-Z0-9_-]{11})/);
  if (watchMatch?.[1]) return watchMatch[1];

  const shortMatch = raw.match(/youtu\.be\/([a-zA-Z0-9_-]{11})/);
  if (shortMatch?.[1]) return shortMatch[1];

  const embedMatch = raw.match(/\/embed\/([a-zA-Z0-9_-]{11})/);
  if (embedMatch?.[1]) return embedMatch[1];

  return null;
}

function normalizeYouTubeUrl(input: string): string | null {
  const id = extractYouTubeId(input);

  if (!id) return null;

  return `https://www.youtube.com/watch?v=${id}`;
}

function flattenQuestions(questions: SectionQuestions): Question[] {
  return [
    ...questions.cloze.map((data) => ({
      id: crypto.randomUUID(),
      type: "cloze" as const,
      data,
    })),
    ...questions.flashcards.map((data) => ({
      id: crypto.randomUUID(),
      type: "flashcard" as const,
      data,
    })),
    ...questions.open_ended.map((data) => ({
      id: crypto.randomUUID(),
      type: "open_ended" as const,
      data,
    })),
  ];
}

export default function InteractiveYouTubeFocusPage() {
  const navigate = useNavigate();

  const [youtubeInput, setYoutubeInput] = useState(
    import.meta.env.DEV
      ? "https://www.youtube.com/watch?v=K5KVEU3aaeQ"
      : ""
  );

  const [status, setStatus] = useState<PlaybackStatus>("idle");
  const [playerReady, setPlayerReady] = useState(false);

  const [lastError, setLastError] = useState<string | null>(null);
  const [questionsCount, setQuestionCount] = useState(0);

  const [questionQueue, setQuestionQueue] = useState<Question[]>([]);
  const [activeQuestionIdx, setActiveQuestionIdx] = useState(0);
  const [reveal, setReveal] = useState(false);
  const [attempt, setAttempt] = useState("");
  const [summaryGenerating, setSummaryGenerating] = useState(false);
  const [summaryError, setSummaryError] = useState<string | null>(null);

  const playerRef = useRef<any>(null);

  // All questions returned by the backend, ordered by trigger_at.
  const scheduledQuestionsRef = useRef<ScheduledQuestion[]>([]);

  // Index of the next scheduled question group.
  const nextScheduledIdxRef = useRef(0);

  const playbackTimerRef = useRef<number | null>(null);

  const videoId = useMemo(
    () => extractYouTubeId(youtubeInput),
    [youtubeInput]
  );

  const normalizedYoutubeUrl = useMemo(
    () => normalizeYouTubeUrl(youtubeInput),
    [youtubeInput]
  );

  const activeQuestion = questionQueue[activeQuestionIdx] ?? null;

  const canStart = Boolean(normalizedYoutubeUrl && playerReady);
  const hasQuestions = questionQueue.length > 0;

  // Authentication.
  useEffect(() => {
    if (!isAuthenticated()) {
      navigate("/login");
    }
  }, [navigate]);

  // Initialize the YouTube IFrame API and player.
  useEffect(() => {
    if (!videoId) {
      setPlayerReady(false);
      return;
    }

    let cancelled = false;

    const createPlayer = () => {
      if (cancelled || !window.YT?.Player) return;

      // Reuse the existing player when the video input changes.
      if (playerRef.current) {
        playerRef.current.cueVideoById(videoId);
        return;
      }

      playerRef.current = new window.YT.Player("yt-player", {
        videoId,
        playerVars: {
          modestbranding: 1,
          rel: 0,
          enablejsapi: 1,
          origin: window.location.origin,
        },
        events: {
          onReady: () => {
            if (!cancelled) {
              setPlayerReady(true);
            }
          },

          onStateChange: (event: any) => {
            // 1 = playing, 2 = paused, 3 = buffering.
            if (event.data === 1) {
              setStatus("playing");
            } else if (event.data === 2 || event.data === 3) {
              setStatus("waiting");
            }
          },

          onError: (event: any) => {
            console.error("YouTube player error:", event.data);
            setLastError("Unable to load this YouTube video.");
          },
        },
      });
    };

    if (window.YT?.Player) {
      createPlayer();
    } else {
      const existingScript = document.querySelector<HTMLScriptElement>(
        'script[data-inker="yt-iframe-api"]'
      );

      if (!existingScript) {
        const script = document.createElement("script");

        script.src = "https://www.youtube.com/iframe_api";
        script.async = true;
        script.dataset.inker = "yt-iframe-api";

        document.body.appendChild(script);
      }

      const previousCallback = window.onYouTubeIframeAPIReady;

      window.onYouTubeIframeAPIReady = () => {
        previousCallback?.();
        createPlayer();
      };
    }

    return () => {
      cancelled = true;
    };
  }, [videoId]);

  // Poll the player's actual playback position and trigger scheduled questions.
  useEffect(() => {
    playbackTimerRef.current = window.setInterval(() => {
      const player = playerRef.current;

      // Only process scheduled questions while the video is playing.
      if (!player || player.getPlayerState?.() !== 1) {
        return;
      }

      const scheduled = scheduledQuestionsRef.current;
      const nextIdx = nextScheduledIdxRef.current;
      const nextGroup = scheduled[nextIdx];

      if (!nextGroup) {
        return;
      }

      const currentTime = player.getCurrentTime();

      if (currentTime < nextGroup.trigger_at) {
        return;
      }

      // Consume this timestamp before pausing the player.
      nextScheduledIdxRef.current++;

      player.pauseVideo();

      const flattenedQuestions = flattenQuestions(nextGroup.questions);

      // No questions in this group: continue playback.
      if (flattenedQuestions.length === 0) {
        player.playVideo();
        return;
      }

      setQuestionQueue(flattenedQuestions);
      setActiveQuestionIdx(0);
      setReveal(false);
      setAttempt("");
    }, 250);

    return () => {
      if (playbackTimerRef.current !== null) {
        window.clearInterval(playbackTimerRef.current);
        playbackTimerRef.current = null;
      }

      playerRef.current?.destroy?.();
      playerRef.current = null;
    };
  }, []);

  async function generateSummary() {
    if (!normalizedYoutubeUrl) {
      setSummaryError("Please enter a valid YouTube URL.");
      return;
    }

    setSummaryError(null);
    setSummaryGenerating(true);

    try {
      const data = await ytSummaryHandler({ body: { url: normalizedYoutubeUrl } })

      // Adjust this path to match your ApiResponse structure.
      navigate(`/notes/${data.data.data.id}`);
    } catch (error) {
      setSummaryError(
        error instanceof Error ? error.message : "Something went wrong."
      );
    } finally {
      setSummaryGenerating(false);
    }
  }
  async function connectAndStart() {
    setLastError(null);
    setReveal(false);
    setAttempt("");

    setQuestionQueue([]);
    setActiveQuestionIdx(0);
    setQuestionCount(0);

    scheduledQuestionsRef.current = [];
    nextScheduledIdxRef.current = 0;

    if (!normalizedYoutubeUrl) {
      setLastError("Please paste a valid YouTube URL or video ID.");
      return;
    }

    if (!playerReady || !playerRef.current) {
      setLastError("The YouTube player is not ready yet.");
      return;
    }

    setStatus("generating");

    try {
      const res = await ytQuestionHandler({
        body: {
          url: normalizedYoutubeUrl,
        }
      });
      const data = res.data
      // Keep the backend's timestamp ordering.
      scheduledQuestionsRef.current = data.questions;
      nextScheduledIdxRef.current = 0;

      const totalQuestions = data.questions.reduce((total, group) => {
        return (
          total +
          group.questions.cloze.length +
          group.questions.flashcards.length +
          group.questions.open_ended.length
        );
      }, 0);

      setQuestionCount(totalQuestions);

      // Start from the beginning after generation finishes.
      playerRef.current.seekTo(0, true);
      playerRef.current.playVideo();

      setStatus("playing");
    } catch (error) {
      console.error("Failed to generate questions:", error);

      setStatus("idle");

      setLastError(
        error instanceof Error
          ? error.message
          : "Something went wrong while generating questions."
      );
    }
  }

  function nextQuestion() {
    setReveal(false);
    setAttempt("");

    if (activeQuestionIdx < questionQueue.length - 1) {
      setActiveQuestionIdx((idx) => idx + 1);
      return;
    }

    // Finished this timestamp's question group.
    setQuestionQueue([]);
    setActiveQuestionIdx(0);

    // Resume playback until the next scheduled timestamp.
    playerRef.current?.playVideo();
  }

  function restartQueue() {
    setReveal(false);
    setAttempt("");
    setActiveQuestionIdx(0);
  }

  return (
    <div className="min-h-screen bg-background bg-gradient-hero">
      <div className="fixed inset-0 bg-grid-pattern opacity-10 pointer-events-none" />

      <header className="sticky top-0 z-50 border-b border-border/50 bg-background/80 backdrop-blur-md">
        <div className="container mx-auto px-4 py-3 flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Button variant="ghost" size="sm" asChild>
              <Link to="/dashboard" className="flex items-center gap-2">
                <ArrowLeft className="w-4 h-4" />
                Back
              </Link>
            </Button>

            <Logo href="/dashboard" size="sm" />
          </div>

          <div className="flex items-center gap-2">
            <div className="text-xs text-muted-foreground hidden sm:block">
              Status:{" "}
              <span
                className={cn(
                  "font-medium",
                  status === "playing" && "text-emerald-500",
                  status === "generating" && "text-amber-500",
                  status === "waiting" && "text-blue-500",
                  status === "idle" && "text-muted-foreground"
                )}
              >
                {status}
              </span>
            </div>

            <Button
              variant="ghost"
              size="sm"
              onClick={() => {
                clearTokens();
                navigate("/login");
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
          {/* Focus Lecture */}
          <Card className="border-border/50 bg-card/50 backdrop-blur-sm">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <span>Focus Lecture</span>

                <div className="text-xs text-muted-foreground">
                  Questions:{" "}
                  <span className="font-medium">{questionsCount}</span>
                </div>
              </CardTitle>
            </CardHeader>

            <CardContent className="space-y-4">
              <div className="space-y-2">
                <div className="text-sm font-medium">
                  YouTube URL / Video ID
                </div>

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
                  disabled={!canStart || status === "generating"}
                  className="h-11"
                >
                  <Play className="w-4 h-4 mr-2" />
                  {status === "generating" ? "Generating..." : "Start"}
                </Button>

                <Button
                  variant="outline"
                  onClick={generateSummary}
                  disabled={!normalizedYoutubeUrl || summaryGenerating}
                  className="h-11"
                >
                  {summaryGenerating ? "Generating Summary..." : "Generate Summary"}
                </Button>
              </div>

              {summaryError && (
                <div className="text-sm text-destructive">{summaryError}</div>
              )}

              <div className="rounded-xl overflow-hidden border border-border/50 bg-black/40">
                {videoId ? (
                  <div className="aspect-video w-full">
                    <div id="yt-player" className="w-full h-full" />
                  </div>
                ) : (
                  <div className="aspect-video flex items-center justify-center text-sm text-muted-foreground">
                    Paste a YouTube link to show the player.
                  </div>
                )}
              </div>

              <div className="text-xs text-muted-foreground">
                This mode intentionally shows{" "}
                <span className="font-medium">questions first</span>. Questions
                appear when their scheduled timestamps are reached.
              </div>
            </CardContent>
          </Card>

          {/* Live Questions */}
          <Card className="border-border/50 bg-card/50 backdrop-blur-sm">
            <CardHeader className="pb-3">
              <CardTitle className="flex items-center justify-between">
                <span>Live Questions</span>

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
              </CardTitle>
            </CardHeader>

            <CardContent className="space-y-4">
              {!activeQuestion ? (
                <div className="rounded-xl border border-dashed border-border/60 bg-background/40 p-6 text-sm text-muted-foreground">
                  {status === "generating"
                    ? "Generating questions..."
                    : status === "playing"
                      ? "Watching lecture. Questions will appear at scheduled timestamps."
                      : "Start a YouTube lecture to generate questions."}
                </div>
              ) : (
                <>
                  <div className="text-xs text-muted-foreground">
                    Question {activeQuestionIdx + 1} / {questionQueue.length} •{" "}
                    {activeQuestion.type.toUpperCase()}
                  </div>

                  <div className="rounded-2xl border border-border/50 bg-background/50 p-5">
                    {/* Question */}
                    <div className="text-lg font-semibold leading-snug">
                      {activeQuestion.type === "cloze"
                        ? activeQuestion.data.text
                        : activeQuestion.data.question}
                    </div>

                    {/* Your answer */}
                    <div className="mt-4 space-y-2">
                      <div className="text-sm font-medium text-muted-foreground">
                        Your attempt
                      </div>

                      <Textarea
                        value={attempt}
                        onChange={(e) => setAttempt(e.target.value)}
                        placeholder="Type your answer..."
                        className="min-h-24 bg-card/50 border-border/50"
                      />
                    </div>

                    {/* Reveal answer */}
                    {reveal && (
                      <div className="mt-4 rounded-xl border border-border/50 bg-card/50 p-4">
                        <div className="text-sm font-medium mb-2">
                          {activeQuestion.type === "flashcard"
                            ? "Flashcard Answer"
                            : activeQuestion.type === "cloze"
                              ? "Correct Answers"
                              : "Expected Answer"}
                        </div>

                        {activeQuestion.type === "cloze" ? (
                          <ul className="list-disc pl-5 text-sm text-foreground">
                            {activeQuestion.data.answers.map((answer, i) => (
                              <li key={i}>{answer}</li>
                            ))}
                          </ul>
                        ) : (
                          <div className="text-sm text-foreground">
                            {activeQuestion.data.answer}
                          </div>
                        )}
                      </div>
                    )}

                    {/* Controls */}
                    <div className="mt-4 flex flex-wrap gap-3">
                      <Button
                        variant={reveal ? "outline" : "default"}
                        onClick={() => setReveal((value) => !value)}
                        className="h-11"
                      >
                        {reveal ? "Hide answer" : "Reveal answer"}
                      </Button>

                      <Button
                        variant="outline"
                        onClick={nextQuestion}
                        className="h-11"
                      >
                        {activeQuestionIdx < questionQueue.length - 1
                          ? "Next"
                          : "Continue video"}
                      </Button>
                    </div>
                  </div>
                </>
              )}

              {/* Queue */}
              {questionQueue.length > 0 && (
                <div className="pt-2 border-t border-border/40">
                  <div className="text-xs text-muted-foreground mb-2">
                    Queue (next up)
                  </div>

                  <div className="space-y-2 max-h-40 overflow-auto pr-2">
                    {questionQueue
                      .slice(activeQuestionIdx + 1, activeQuestionIdx + 6)
                      .map((question) => (
                        <div
                          key={question.id}
                          className="text-sm text-muted-foreground truncate"
                        >
                          {question.type === "cloze"
                            ? question.data.text
                            : question.data.question}
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
