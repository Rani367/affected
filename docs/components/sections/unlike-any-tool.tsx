import { Container } from "../container";

const sizeComparison = [
  { name: "affected", mb: 5, color: "#00ff66" },
  { name: "Turborepo", mb: 100, color: "#858699" },
  { name: "Nx", mb: 200, color: "#858699" },
  { name: "Bazel", mb: 500, color: "#858699" },
];

const ciPlatforms = [
  { name: "GitHub Actions", color: "#ffffff" },
  { name: "GitLab CI", color: "#fc6d26" },
  { name: "CircleCI", color: "#5eacd3" },
  { name: "Azure Pipelines", color: "#0078d4" },
  { name: "Jenkins", color: "#d33833" },
];

export const UnlikeAnyTool = () => (
  <div className="text-white">
    <Container>
      <div className="text-center">
        <h2 className="mb-4 text-4xl md:mb-7 md:text-7xl">
          Built different.
        </h2>
        <p className="mx-auto mb-12 max-w-[68rem] text-lg text-primary-text md:mb-7 md:text-xl">
          Zero config. Zero dependencies. Just one binary that understands your
          entire monorepo.
        </p>
      </div>
    </Container>
    <div className="h-[48rem] overflow-hidden md:h-auto md:overflow-auto">
      <div className="flex snap-x snap-mandatory gap-6 overflow-x-auto px-8 pb-12 md:flex-wrap md:overflow-hidden">

        {/* Card 1: Zero Configuration — 2/3 width */}
        <div className="relative flex min-h-[48rem] w-full shrink-0 snap-center flex-col items-center justify-end overflow-hidden rounded-[4.8rem] border border-transparent-white bg-glass-gradient p-8 text-center md:max-w-[calc(66.66%-12px)] md:basis-[calc(66.66%-12px)] md:p-14">
          <div className="mb-8 w-full max-w-[48rem] overflow-hidden rounded-[1.2rem] border border-transparent-white bg-[rgba(255,255,255,0.03)]">
            {/* Window chrome */}
            <div className="flex items-center gap-[6px] border-b border-transparent-white bg-[rgba(255,255,255,0.02)] px-4 py-[10px]">
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#ff5f57]" />
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#febc2e]" />
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#28c840]" />
              <span className="ml-3 font-mono text-xs text-grey">affected — zsh</span>
            </div>
            <div className="p-6 text-left font-mono text-sm leading-relaxed">
              <div>
                <span style={{ color: "#858699" }}>$</span>{" "}
                <span className="text-white">affected test --base main</span>
              </div>
              <div className="mt-1 text-primary-text">
                Detected ecosystem: Cargo workspace
              </div>
              <div className="text-primary-text">
                Found 23 packages, 4 affected
              </div>
              <div style={{ color: "#00ff66" }}>
                Running: cargo test -p core -p api -p cli -p utils
              </div>
              <div style={{ color: "#00ff66" }}>
                ✓ All 4 tests passed · 3.1s
              </div>
            </div>
          </div>
          <p className="mb-4 text-3xl">Zero Configuration</p>
          <p className="text-md text-primary-text">
            Auto-detects your ecosystem from manifest files. Cargo.toml? Rust
            workspace. package.json? Node monorepo. No setup required.
          </p>
        </div>

        {/* Card 2: Lightning Fast — 1/3 width */}
        <div className="relative flex min-h-[48rem] w-full shrink-0 snap-center flex-col items-center justify-end overflow-hidden rounded-[4.8rem] border border-transparent-white bg-glass-gradient p-8 text-center md:basis-[calc(33.33%-12px)] md:p-14">
          <div className="mb-8 w-full">
            {/* Size comparison bars */}
            <div className="mb-7 w-full space-y-[10px]">
              {sizeComparison.map(({ name, mb, color }) => (
                <div key={name} className="flex items-center gap-3">
                  <span
                    className="w-[6.5rem] shrink-0 text-right text-xs"
                    style={{ color: color === "#858699" ? "#858699" : color }}
                  >
                    {name}
                  </span>
                  <div className="flex-1 overflow-hidden rounded-full bg-[rgba(255,255,255,0.06)]" style={{ height: 5 }}>
                    <div
                      className="h-full rounded-full"
                      style={{
                        width: `${(mb / 500) * 100}%`,
                        background: color,
                        opacity: color === "#858699" ? 0.45 : 1,
                      }}
                    />
                  </div>
                  <span
                    className="w-[4.5rem] text-left text-xs"
                    style={{ color: color === "#858699" ? "#858699" : color }}
                  >
                    {mb < 50 ? `~${mb} MB` : `~${mb}+ MB`}
                  </span>
                </div>
              ))}
            </div>

            <span className="text-[5rem] font-bold leading-none text-white opacity-90">
              ~5
              <span className="text-3xl text-primary-text">MB</span>
            </span>
            <div className="mt-3 flex items-center justify-center gap-3 text-sm text-primary-text">
              <span className="inline-block h-2 w-2 rounded-full bg-white" />
              Single binary
              <span className="inline-block h-2 w-2 rounded-full bg-grey" />
              No runtime
            </div>
          </div>
          <p className="mb-4 text-3xl">Lightning Fast</p>
          <p className="text-md text-primary-text">
            Single ~5MB Rust binary. No Node.js, no JVM, no runtime
            dependencies. Starts in milliseconds.
          </p>
        </div>

        {/* Card 3: Full Blast Radius — 1/3 width */}
        <div className="relative flex min-h-[48rem] w-full shrink-0 snap-center flex-col items-center justify-end overflow-hidden rounded-[4.8rem] border border-transparent-white bg-glass-gradient p-8 text-center md:basis-[calc(33.33%-12px)] md:p-14">
          <div className="mb-8 flex flex-col items-center gap-2">
            {/* Changed node */}
            <div className="flex items-center gap-2">
              <span
                className="rounded px-[7px] py-[2px] text-[10px] uppercase tracking-widest font-medium"
                style={{
                  color: "#febc2e",
                  background: "rgba(249,173,0,0.08)",
                  border: "1px solid rgba(249,173,0,0.25)",
                }}
              >
                changed
              </span>
            </div>
            <div
              className="rounded-[0.6rem] border px-4 py-1.5 text-sm font-semibold"
              style={{
                color: "#febc2e",
                borderColor: "rgba(249,173,0,0.45)",
                background: "rgba(249,173,0,0.08)",
              }}
            >
              core
            </div>

            {/* Connector */}
            <svg width="12" height="28" viewBox="0 0 12 28" aria-hidden>
              <line x1="6" y1="0" x2="6" y2="28" stroke="rgba(249,173,0,0.3)" strokeWidth="1" strokeDasharray="3 2.5" />
            </svg>

            {/* Affected nodes */}
            <div className="flex items-center gap-2">
              {["api", "cli", "web"].map((pkg) => (
                <div key={pkg} className="flex flex-col items-center gap-[5px]">
                  <div
                    className="rounded-[0.6rem] border px-3 py-1 text-sm"
                    style={{
                      color: "#00ff66",
                      borderColor: "rgba(0,255,102,0.32)",
                      background: "rgba(0,255,102,0.06)",
                    }}
                  >
                    {pkg}
                  </div>
                  <span
                    className="text-[9px] uppercase tracking-wider"
                    style={{ color: "#00ff66", opacity: 0.55 }}
                  >
                    affected
                  </span>
                </div>
              ))}
            </div>
          </div>
          <p className="mb-4 text-3xl">Full Blast Radius</p>
          <p className="text-md text-primary-text">
            Transitive dependency graph analysis. If core changes and api depends
            on it, both are affected.
          </p>
        </div>

        {/* Card 4: CI-Native — 2/3 width */}
        <div className="relative flex min-h-[48rem] w-full shrink-0 snap-center flex-col items-center justify-start overflow-hidden rounded-[4.8rem] border border-transparent-white bg-glass-gradient p-8 text-center md:max-w-[calc(66.66%-12px)] md:basis-[calc(66.66%-12px)] md:p-14">
          <div className="mb-6 w-full max-w-[48rem] overflow-hidden rounded-[1.2rem] border border-transparent-white bg-[rgba(255,255,255,0.03)]">
            {/* Window chrome */}
            <div className="flex items-center gap-[6px] border-b border-transparent-white bg-[rgba(255,255,255,0.02)] px-4 py-[10px]">
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#ff5f57]" />
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#febc2e]" />
              <span className="inline-block h-[9px] w-[9px] rounded-full bg-[#28c840]" />
              <span className="ml-3 font-mono text-xs text-grey">.github/workflows/ci.yml</span>
            </div>
            <div className="p-6 text-left font-mono text-xs leading-relaxed">
              <span className="text-primary-text"># .github/workflows/ci.yml</span>
              <br />
              <span className="text-white">- uses:</span>{" "}
              <span style={{ color: "#00add8" }}>
                Rani367/setup-affected@v1
              </span>
              <br />
              <span className="text-white">- run:</span>{" "}
              <span style={{ color: "#f9ad00" }}>
                affected test --base ${"${{ github.event.pull_request.base.sha }}"}
              </span>
              <br />
              <span className="text-white">- run:</span>{" "}
              <span style={{ color: "#f9ad00" }}>
                affected lint --output junit
              </span>
            </div>
          </div>

          {/* CI platform badges */}
          <div className="mb-auto flex flex-wrap justify-center gap-2">
            {ciPlatforms.map(({ name, color }) => (
              <span
                key={name}
                className="rounded-full border px-3 py-[5px] text-xs font-medium"
                style={{
                  color,
                  borderColor: `${color}2e`,
                  background: `${color}0a`,
                }}
              >
                {name}
              </span>
            ))}
          </div>

          <p className="mb-4 text-3xl">CI-Native</p>
          <p className="text-md text-primary-text">
            GitHub Actions, GitLab CI, CircleCI, Azure Pipelines. Dynamic
            matrices, PR comment bot, JUnit output.
          </p>
        </div>

      </div>
    </div>
  </div>
);
