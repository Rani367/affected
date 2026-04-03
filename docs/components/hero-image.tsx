"use client";

import classNames from "classnames";
import { CSSProperties, useEffect, useRef, useState } from "react";
import { useInView } from "react-intersection-observer";

const randomNumberBetween = (min: number, max: number) => {
  return Math.floor(Math.random() * (max - min + 1) + min);
};

interface Line {
  id: string;
  direction: "to top" | "to left";
  size: number;
  duration: number;
}

interface Demo {
  label: string;
  command: string;
  output: string[];
}

const demos: Demo[] = [
  {
    label: "--explain",
    command: "affected list --base main --explain",
    output: [
      "",
      "3 affected package(s) (base: main, 2 files changed):",
      "",
      "  ● core       (directly changed: src/lib.rs)",
      "  ● api        (depends on: core)",
      "  ● cli        (depends on: api → core)",
    ],
  },
  {
    label: "test",
    command: "affected test --base main --jobs 4",
    output: [
      "",
      "Running tests for 3 affected package(s) (out of 8 total):",
      "",
      "  ▶ core    ✓ passed  (0.8s)",
      "  ▶ api     ✓ passed  (1.2s)",
      "  ▶ cli     ✓ passed  (0.6s)",
      "",
      "All 3 packages passed in 1.4s (parallel)",
    ],
  },
  {
    label: "graph",
    command: "affected graph --base main",
    output: [
      "",
      "Dependency Graph (5 packages, 3 affected):",
      "",
      "  cli  ●",
      "  └── api  ●",
      "      └── core  ●",
      "  utils",
      "  standalone  (no dependencies)",
    ],
  },
];

function colorizeOutput(line: string): React.ReactNode {
  if (line === "") return "\u00A0";

  const parts: React.ReactNode[] = [];
  let key = 0;

  const patterns: [RegExp, string][] = [
    [/●/g, "text-[#00f0ff]"],
    [/✓ passed/g, "text-[#00ff66]"],
    [/└──/g, "text-grey"],
    [/\(\d+\.\d+s\)/g, "text-grey"],
    [/\(parallel\)/g, "text-grey"],
    [/depends on:/g, "text-[#ff0055]"],
    [/directly changed:/g, "text-[#ffcc00]"],
    [/\(no dependencies\)/g, "text-grey"],
  ];

  const spans: { start: number; end: number; className: string }[] = [];

  for (const [pattern, className] of patterns) {
    const regex = new RegExp(pattern.source, "g");
    let match;
    while ((match = regex.exec(line)) !== null) {
      spans.push({ start: match.index, end: match.index + match[0].length, className });
    }
  }

  spans.sort((a, b) => a.start - b.start);

  let pos = 0;
  for (const span of spans) {
    if (span.start > pos) {
      parts.push(<span key={key++}>{line.slice(pos, span.start)}</span>);
    }
    parts.push(
      <span key={key++} className={span.className}>
        {line.slice(span.start, span.end)}
      </span>
    );
    pos = span.end;
  }
  if (pos < line.length) {
    parts.push(<span key={key++}>{line.slice(pos)}</span>);
  }

  return parts.length > 0 ? parts : line;
}

function colorizeCommand(command: string): React.ReactNode {
  const parts: React.ReactNode[] = [];
  const tokens = command.split(" ");
  let key = 0;

  for (let i = 0; i < tokens.length; i++) {
    const token = tokens[i];
    if (i > 0) parts.push(<span key={key++}> </span>);

    if (token === "affected") {
      parts.push(
        <span key={key++} className="text-[#00f0ff]">
          {token}
        </span>
      );
    } else if (token.startsWith("--")) {
      parts.push(
        <span key={key++} className="text-[#ffcc00]">
          {token}
        </span>
      );
    } else {
      parts.push(<span key={key++}>{token}</span>);
    }
  }

  return parts;
}

export const HeroImage = () => {
  const { ref, inView } = useInView({ threshold: 0.4, triggerOnce: true });
  const [lines, setLines] = useState<Line[]>([]);
  const timeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const [demoIndex, setDemoIndex] = useState(0);
  const [typedChars, setTypedChars] = useState(0);
  const [visibleOutputLines, setVisibleOutputLines] = useState(0);
  const [phase, setPhase] = useState<"typing" | "output" | "waiting">("typing");
  const animationTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const removeLine = (id: string) => {
    setLines((prev) => prev.filter((line) => line.id !== id));
  };

  useEffect(() => {
    if (!inView) return;

    const renderLine = (timeout: number) => {
      timeoutRef.current = setTimeout(() => {
        setLines((lines) => [
          ...lines,
          {
            direction: Math.random() > 0.5 ? "to top" : "to left",
            duration: randomNumberBetween(1300, 3500),
            size: randomNumberBetween(10, 30),
            id: Math.random().toString(36).substring(7),
          },
        ]);
        renderLine(randomNumberBetween(800, 2500));
      }, timeout);
    };

    renderLine(randomNumberBetween(800, 1300));

    return () => {
      if (timeoutRef.current) clearTimeout(timeoutRef.current);
    };
  }, [inView, setLines]);

  useEffect(() => {
    if (!inView) return;

    const demo = demos[demoIndex];

    if (phase === "typing") {
      if (typedChars < demo.command.length) {
        const jitter = randomNumberBetween(20, 50);
        animationTimeoutRef.current = setTimeout(() => {
          setTypedChars((c) => c + 1);
        }, jitter);
      } else {
        animationTimeoutRef.current = setTimeout(() => {
          setPhase("output");
          setVisibleOutputLines(0);
        }, 300);
      }
    } else if (phase === "output") {
      if (visibleOutputLines < demo.output.length) {
        animationTimeoutRef.current = setTimeout(() => {
          setVisibleOutputLines((v) => v + 1);
        }, 80);
      } else {
        animationTimeoutRef.current = setTimeout(() => {
          setPhase("waiting");
        }, 3500);
      }
    } else if (phase === "waiting") {
      setDemoIndex((i) => (i + 1) % demos.length);
      setTypedChars(0);
      setVisibleOutputLines(0);
      setPhase("typing");
    }

    return () => {
      if (animationTimeoutRef.current) clearTimeout(animationTimeoutRef.current);
    };
  }, [inView, demoIndex, typedChars, visibleOutputLines, phase]);

  const currentDemo = demos[demoIndex];
  const displayedCommand = currentDemo.command.slice(0, typedChars);
  const showCursor = phase === "typing";

  const handleTabClick = (i: number) => {
    if (animationTimeoutRef.current) clearTimeout(animationTimeoutRef.current);
    setDemoIndex(i);
    setTypedChars(0);
    setVisibleOutputLines(0);
    setPhase("typing");
  };

  return (
    <div ref={ref} className="mt-[12.8rem] [perspective:2000px]">
      <div
        className={classNames(
          "relative rounded-lg border border-transparent-white bg-white bg-opacity-[0.01] bg-hero-gradient",
          inView ? "animate-image-rotate" : "[transform:rotateX(25deg)]",
          "before:absolute before:top-0 before:left-0 before:h-full before:w-full before:bg-hero-glow before:opacity-0 before:[filter:blur(120px)]",
          inView && "before:animate-image-glow"
        )}
      >
        <div className="absolute top-0 left-0 z-20 h-full w-full">
          {lines.map((line) => (
            <span
              key={line.id}
              onAnimationEnd={() => removeLine(line.id)}
              style={
                {
                  "--direction": line.direction,
                  "--size": line.size,
                  "--animation-duration": `${line.duration}ms`,
                } as CSSProperties
              }
              className={classNames(
                "absolute top-0 block h-[1px] w-[10rem] bg-glow-lines",
                line.direction === "to left" &&
                  `left-0 h-[1px] w-[calc(var(--size)*0.5rem)] animate-glow-line-horizontal md:w-[calc(var(--size)*1rem)]`,
                line.direction === "to top" &&
                  `right-0 h-[calc(var(--size)*0.5rem)] w-[1px] animate-glow-line-vertical md:h-[calc(var(--size)*1rem)]`
              )}
            />
          ))}
        </div>
        <svg
          className={classNames(
            "absolute left-0 top-0 h-full w-full",
            "[&_path]:stroke-white [&_path]:[strokeOpacity:0.15] [&_path]:[stroke-dasharray:1] [&_path]:[stroke-dashoffset:1]",
            inView && "[&_path]:animate-sketch-lines"
          )}
          width="100%"
          viewBox="0 0 1499 778"
          fill="none"
        >
          <path pathLength="1" d="M1500 72L220 72"></path>
          <path pathLength="1" d="M1500 128L220 128"></path>
          <path pathLength="1" d="M1500 189L220 189"></path>
          <path pathLength="1" d="M220 777L220 1"></path>
          <path pathLength="1" d="M538 777L538 128"></path>
        </svg>

        {/* Terminal window */}
        <div
          className={classNames(
            "relative z-10 transition-opacity delay-[680ms]",
            inView ? "opacity-100" : "opacity-0"
          )}
        >
          <div className="overflow-hidden rounded-lg border border-transparent-white bg-[#0d0f14]">
            {/* Terminal header with demo tabs */}
            <div className="flex items-center border-b border-transparent-white bg-[rgba(255,255,255,0.03)] px-[1.6rem] py-[1.2rem]">
              <div className="flex gap-[0.8rem]">
                <div className="h-[1.2rem] w-[1.2rem] rounded-full bg-[#ff5f57]" />
                <div className="h-[1.2rem] w-[1.2rem] rounded-full bg-[#febc2e]" />
                <div className="h-[1.2rem] w-[1.2rem] rounded-full bg-[#28c840]" />
              </div>

              {/* Clickable demo tabs */}
              <div className="flex flex-1 items-center justify-center gap-1">
                {demos.map((demo, i) => (
                  <button
                    key={i}
                    onClick={() => handleTabClick(i)}
                    className={classNames(
                      "rounded px-[12px] py-[4px] font-mono text-[1.3rem] tracking-wide transition-colors duration-150",
                      i === demoIndex
                        ? "bg-[rgba(0,240,255,0.12)] text-[#00f0ff]"
                        : "text-grey hover:text-white"
                    )}
                  >
                    {demo.label}
                  </button>
                ))}
              </div>

              <div className="w-[5.2rem]" />
            </div>

            {/* Terminal body */}
            <div className="min-h-[32rem] p-[2.4rem] font-mono text-[1.5rem] leading-[2.4rem] text-white/85">
              <div className="whitespace-pre flex items-center gap-[0.4rem]">
                <span className="text-[#7c6af7]">~/projects/myapp</span>
                <span className="text-white/30 mx-[0.3rem]">on</span>
                <span className="text-[#f97583]"> main</span>
                <span className="text-white/30 mx-[0.3rem]">›</span>
                <span className="text-[#00f0ff]">$</span>
                <span> </span>
                {colorizeCommand(displayedCommand)}
                {showCursor && (
                  <span className="inline-block h-[1.6rem] w-[0.7rem] animate-[cursor-blink_1s_step-end_infinite] align-middle bg-white/70" />
                )}
              </div>

              {phase !== "typing" &&
                currentDemo.output.slice(0, visibleOutputLines).map((outputLine, i) => (
                  <div key={i} className="whitespace-pre text-white/75">
                    {colorizeOutput(outputLine)}
                  </div>
                ))}
            </div>

            {/* Demo progress indicator */}
            <div className="flex items-center justify-center gap-[6px] pb-[1.4rem]">
              {demos.map((_, i) => (
                <button
                  key={i}
                  onClick={() => handleTabClick(i)}
                  className="rounded-full transition-all duration-500"
                  style={{
                    height: 4,
                    width: i === demoIndex ? 20 : 6,
                    background: i === demoIndex ? "#00f0ff" : "rgba(255,255,255,0.18)",
                  }}
                  aria-label={`Switch to ${demos[i].label} demo`}
                />
              ))}
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};
