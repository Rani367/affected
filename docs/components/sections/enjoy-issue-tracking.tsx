"use client";

import { Features } from "../features";
import { Container } from "../container";

const steps = [
  {
    number: "01",
    title: "Detect.",
    text: "Scans for manifest files to identify your ecosystem.",
  },
  {
    number: "02",
    title: "Resolve.",
    text: "Parses manifests into a full dependency graph.",
  },
  {
    number: "03",
    title: "Diff.",
    text: "Computes changed files via libgit2.",
  },
  {
    number: "04",
    title: "Map.",
    text: "Maps each changed file to its owning package.",
  },
  {
    number: "05",
    title: "Traverse.",
    text: "Reverse BFS to find all dependents.",
  },
  {
    number: "06",
    title: "Execute.",
    text: "Runs your command on only the affected packages.",
  },
];

export const EnjoyIssueTracking = () => {
  return (
    <Features color="0,240,255" colorDark="0,136,170">
      <Features.Main
        title={
          <>
            How it
            <br />
            works
          </>
        }
        image=""
        text="From git diff to targeted test execution — six steps, milliseconds. affected builds a dependency graph of your monorepo, diffs against your base branch, and runs commands on only the affected packages."
      />

      {/* Numbered steps replacing icon grid */}
      <Container>
        <div className="mb-16 grid w-full grid-cols-2 gap-x-8 gap-y-10 text-sm text-primary-text md:mb-[14rem] md:grid-cols-3 md:text-md">
          {steps.map(({ number, title, text }) => (
            <div key={number} className="flex flex-col gap-3">
              <div
                className="w-fit rounded-[0.6rem] px-[9px] py-[3px] font-mono text-xs font-bold tracking-wider"
                style={{
                  color: "rgb(var(--feature-color))",
                  background: "rgba(var(--feature-color), 0.08)",
                  border: "1px solid rgba(var(--feature-color), 0.2)",
                }}
              >
                {number}
              </div>
              <div>
                <span className="text-white">{title}</span>{" "}
                {text}
              </div>
            </div>
          ))}
        </div>
      </Container>

      {/* Inline cards replacing Features.Cards */}
      <Container>
        <div className="grid w-full grid-cols-1 gap-6 md:grid-cols-2">

          {/* --explain card */}
          <div className="relative overflow-hidden rounded-[2.4rem] border border-transparent-white bg-[radial-gradient(ellipse_at_center,rgba(var(--feature-color),0.12),transparent)] px-8 py-8 before:pointer-events-none before:absolute before:inset-0 before:bg-glass-gradient md:rounded-[4.8rem] md:p-14">
            {/* Illustration: dependency chain output */}
            <div
              className="mb-8 overflow-hidden rounded-[0.8rem] border border-transparent-white bg-[rgba(255,255,255,0.03)] p-5"
              style={{
                fontFamily: "ui-monospace, 'SF Mono', Consolas, monospace",
                fontSize: "1.2rem",
                lineHeight: 1.9,
              }}
            >
              <div>
                <span style={{ color: "#858699" }}>$</span>{" "}
                <span style={{ color: "rgb(var(--feature-color))" }}>affected</span>
                <span className="text-white"> list --base main --explain</span>
              </div>
              <div className="mt-1 text-primary-text">
                3 affected package(s):
              </div>
              <div>
                <span style={{ color: "rgb(var(--feature-color))" }}>●</span>
                <span className="text-white"> core</span>
                <span style={{ color: "#febc2e" }}> (directly changed: src/lib.rs)</span>
              </div>
              <div>
                <span style={{ color: "rgb(var(--feature-color))" }}>●</span>
                <span className="text-white"> api</span>
                <span style={{ color: "#858699" }}> (depends on: </span>
                <span style={{ color: "#ff0055" }}>core</span>
                <span style={{ color: "#858699" }}>)</span>
              </div>
              <div>
                <span style={{ color: "rgb(var(--feature-color))" }}>●</span>
                <span className="text-white"> cli</span>
                <span style={{ color: "#858699" }}> (depends on: </span>
                <span style={{ color: "#ff0055" }}>api → core</span>
                <span style={{ color: "#858699" }}>)</span>
              </div>
            </div>
            <h3 className="mb-2 text-2xl text-white">The --explain flag</h3>
            <p className="max-w-[31rem] text-md text-primary-text">
              See exactly why each package is affected. Full dependency chain
              visualization from changed file to impacted package.
            </p>
          </div>

          {/* Watch mode card */}
          <div className="relative overflow-hidden rounded-[2.4rem] border border-transparent-white bg-[radial-gradient(ellipse_at_center,rgba(var(--feature-color),0.12),transparent)] px-8 py-8 before:pointer-events-none before:absolute before:inset-0 before:bg-glass-gradient md:rounded-[4.8rem] md:p-14">
            {/* Illustration: watch mode output */}
            <div
              className="mb-8 overflow-hidden rounded-[0.8rem] border border-transparent-white bg-[rgba(255,255,255,0.03)] p-5"
              style={{
                fontFamily: "ui-monospace, 'SF Mono', Consolas, monospace",
                fontSize: "1.2rem",
                lineHeight: 1.9,
              }}
            >
              <div>
                <span style={{ color: "#858699" }}>$</span>{" "}
                <span style={{ color: "rgb(var(--feature-color))" }}>affected</span>
                <span className="text-white"> test --watch --base main</span>
              </div>
              <div className="mt-1" style={{ color: "rgb(var(--feature-color))" }}>
                watching 3 packages...
              </div>
              <div style={{ color: "#858699" }}>──────────────────────</div>
              <div>
                <span style={{ color: "#febc2e" }}>~</span>
                <span className="text-white"> src/core/mod.rs</span>
                <span style={{ color: "#febc2e" }}> modified</span>
              </div>
              <div>
                <span style={{ color: "rgb(var(--feature-color))" }}>↺</span>
                <span className="text-white"> re-running: cargo test -p core</span>
              </div>
              <div>
                <span style={{ color: "#00ff66" }}>✓</span>
                <span className="text-white"> core</span>
                <span style={{ color: "#858699" }}> passed (0.6s)</span>
              </div>
            </div>
            <h3 className="mb-2 text-2xl text-white">Watch mode</h3>
            <p className="max-w-[31rem] text-md text-primary-text">
              Re-runs on file change. Built-in debouncing for smooth dev loops.
              Your tests stay in sync as you code.
            </p>
          </div>

        </div>
      </Container>
    </Features>
  );
};
