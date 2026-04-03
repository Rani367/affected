"use client";

import { Features } from "../features";
import { Container } from "../container";

const comparisonData = [
  {
    feature: "Zero config",
    affected: { value: "\u2713", highlight: true },
    nx: { value: "\u2717", highlight: false },
    turborepo: { value: "\u2717", highlight: false },
    bazel: { value: "\u2717", highlight: false },
  },
  {
    feature: "Standalone binary",
    affected: { value: "\u2713", highlight: true },
    nx: { value: "Node.js", highlight: false },
    turborepo: { value: "Node.js", highlight: false },
    bazel: { value: "JVM", highlight: false },
  },
  {
    feature: "Setup time",
    affected: { value: "1 min", highlight: true },
    nx: { value: "Hours", highlight: false },
    turborepo: { value: "Hours", highlight: false },
    bazel: { value: "Days", highlight: false },
  },
  {
    feature: "Binary size",
    affected: { value: "~5 MB", highlight: true },
    nx: { value: "~200 MB+", highlight: false },
    turborepo: { value: "~100 MB+", highlight: false },
    bazel: { value: "~500 MB+", highlight: false },
  },
  {
    feature: "Ecosystems",
    affected: { value: "13", highlight: true },
    nx: { value: "JS/TS", highlight: false },
    turborepo: { value: "JS/TS", highlight: false },
    bazel: { value: "Any", highlight: false },
  },
  {
    feature: "--explain",
    affected: { value: "\u2713", highlight: true },
    nx: { value: "\u2717", highlight: false },
    turborepo: { value: "\u2717", highlight: false },
    bazel: { value: "\u2717", highlight: false },
  },
  {
    feature: "Watch mode",
    affected: { value: "\u2713", highlight: true },
    nx: { value: "\u2713", highlight: false },
    turborepo: { value: "\u2717", highlight: false },
    bazel: { value: "\u2717", highlight: false },
  },
  {
    feature: "Multi-CI",
    affected: { value: "5 platforms", highlight: true },
    nx: { value: "GitHub", highlight: false },
    turborepo: { value: "GitHub", highlight: false },
    bazel: { value: "Custom", highlight: false },
  },
];

function CellValue({ value, highlight }: { value: string; highlight: boolean }) {
  if (value === "\u2713") {
    return (
      <span style={{ color: "#00ff66" }} className={highlight ? "font-medium" : ""}>
        {value}
      </span>
    );
  }
  if (value === "\u2717") {
    return (
      <span style={{ color: "#ff0055", opacity: 0.55 }}>{value}</span>
    );
  }
  return (
    <span className={highlight ? "font-semibold text-white" : "text-primary-text"}>
      {value}
    </span>
  );
}

const ACCENT = "255,0,85";

export const BuildMomentum = () => {
  return (
    <Features color={ACCENT} colorDark="170,0,51">
      <Features.Main
        title={
          <>
            How affected
            <br />
            stacks up
          </>
        }
        image=""
        text="The power of a build system. The simplicity of a CLI."
      />
      <Container>
        <div className="mb-16 w-full overflow-x-auto md:mb-[14rem]">
          <div className="relative min-w-[60rem] overflow-hidden rounded-[2.4rem] border border-transparent-white bg-[rgba(255,255,255,0.03)]">

            {/* Header row */}
            <div className="grid grid-cols-5 border-b border-transparent-white text-sm font-medium text-white">
              <div className="px-8 py-5 text-primary-text">Feature</div>

              {/* Affected column header — highlighted */}
              <div className="relative px-8 py-5 font-semibold" style={{ color: `rgb(${ACCENT})` }}>
                {/* Top accent bar */}
                <span
                  className="pointer-events-none absolute inset-x-0 top-0 h-[2px]"
                  style={{ background: `linear-gradient(to right, transparent, rgba(${ACCENT},0.8), transparent)` }}
                />
                {/* Column bg */}
                <span
                  className="pointer-events-none absolute inset-0"
                  style={{ background: `rgba(${ACCENT},0.06)` }}
                />
                <span className="relative">affected ✦</span>
              </div>

              <div className="px-8 py-5 text-primary-text">Nx</div>
              <div className="px-8 py-5 text-primary-text">Turborepo</div>
              <div className="px-8 py-5 text-primary-text">Bazel</div>
            </div>

            {/* Data rows */}
            {comparisonData.map((row, idx) => (
              <div
                key={row.feature}
                className="grid grid-cols-5 border-b border-transparent-white text-sm last:border-b-0"
              >
                <div className="px-8 py-4 text-white">{row.feature}</div>

                {/* Affected column cell — highlighted */}
                <div className="relative px-8 py-4">
                  <span
                    className="pointer-events-none absolute inset-0"
                    style={{ background: `rgba(${ACCENT},0.04)` }}
                  />
                  {/* Bottom border continuation */}
                  {idx < comparisonData.length - 1 && (
                    <span
                      className="pointer-events-none absolute inset-x-0 bottom-0 h-[1px]"
                      style={{ background: `rgba(${ACCENT},0.1)` }}
                    />
                  )}
                  <span className="relative">
                    <CellValue {...row.affected} />
                  </span>
                </div>

                <div className="px-8 py-4"><CellValue {...row.nx} /></div>
                <div className="px-8 py-4"><CellValue {...row.turborepo} /></div>
                <div className="px-8 py-4"><CellValue {...row.bazel} /></div>
              </div>
            ))}
          </div>
        </div>
      </Container>
    </Features>
  );
};
