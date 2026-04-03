const ecosystems = [
  { name: "Cargo", color: "#dea584" },
  { name: "npm", color: "#cb3837" },
  { name: "pnpm", color: "#f9ad00" },
  { name: "Yarn", color: "#2c8ebb" },
  { name: "Bun", color: "#fbf0df" },
  { name: "Go", color: "#00add8" },
  { name: "Python", color: "#3776ab" },
  { name: "Maven", color: "#e76f00" },
  { name: "Gradle", color: "#1ba8a1" },
  { name: ".NET", color: "#512bd4" },
  { name: "Swift", color: "#f05138" },
  { name: "Dart", color: "#0175c2" },
  { name: "Elixir", color: "#6e4a7e" },
];

function EcosystemIcon({ name, color }: { name: string; color: string }) {
  switch (name) {
    case "Cargo":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <circle cx="7" cy="7" r="2.5" fill={color} />
          <circle cx="7" cy="7" r="5.5" stroke={color} strokeWidth="1.2" strokeDasharray="2.6 1.6" />
        </svg>
      );
    case "npm":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden style={{ flexShrink: 0 }}>
          <rect x="1" y="4" width="12" height="6.5" rx="1" fill={color} />
          <rect x="5.5" y="4" width="3" height="3.5" rx="0.5" fill="#000212" />
        </svg>
      );
    case "pnpm":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden style={{ flexShrink: 0 }}>
          <rect x="0.5" y="0.5" width="4" height="4" rx="0.8" fill={color} />
          <rect x="5" y="0.5" width="4" height="4" rx="0.8" fill={color} opacity="0.65" />
          <rect x="9.5" y="0.5" width="4" height="4" rx="0.8" fill={color} opacity="0.35" />
          <rect x="0.5" y="5.2" width="4" height="4" rx="0.8" fill={color} opacity="0.65" />
          <rect x="5" y="5.2" width="4" height="4" rx="0.8" fill={color} />
          <rect x="0.5" y="9.5" width="4" height="4" rx="0.8" fill={color} opacity="0.35" />
        </svg>
      );
    case "Yarn":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <circle cx="7" cy="7" r="5.5" stroke={color} strokeWidth="1.2" />
          <path d="M3.2 4.8 Q7 7.2 10.8 4.8" stroke={color} strokeWidth="1" />
          <path d="M1.8 7.8 Q7 11 12.2 7.6" stroke={color} strokeWidth="1" />
        </svg>
      );
    case "Bun":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <circle cx="7" cy="7" r="3.2" fill={color} opacity="0.8" />
          <circle cx="7" cy="7" r="5.5" stroke={color} strokeWidth="0.9" strokeDasharray="1.6 2.8" />
        </svg>
      );
    case "Go":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden style={{ flexShrink: 0 }}>
          <ellipse cx="7" cy="8.5" rx="5.5" ry="4" fill={color} opacity="0.75" />
          <circle cx="4.5" cy="8" r="1.1" fill="#000212" />
          <circle cx="9.5" cy="8" r="1.1" fill="#000212" />
          <circle cx="4.9" cy="7.6" r="0.4" fill="white" />
          <circle cx="9.9" cy="7.6" r="0.4" fill="white" />
          <path d="M7 4.5 Q9 3.5 10 4.5" stroke={color} strokeWidth="1.2" fill="none" strokeLinecap="round" />
        </svg>
      );
    case "Python":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M7 1.5 C4.5 1.5 3.5 2.8 3.5 4.5L3.5 5.5L7 5.5L7 6.5L2.5 6.5L2.5 8.5C2.5 10.5 3.8 12 7 12" stroke={color} strokeWidth="1.4" strokeLinecap="round" />
          <path d="M7 12 C9.5 12 10.5 10.7 10.5 9L10.5 8L7 8L7 7L11.5 7L11.5 5C11.5 3 10.2 1.5 7 1.5" stroke={color} strokeWidth="1.4" strokeLinecap="round" />
          <circle cx="5.5" cy="3.8" r="0.9" fill={color} />
          <circle cx="8.5" cy="9.7" r="0.9" fill={color} />
        </svg>
      );
    case "Maven":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M1.5 12 L1.5 2 L5 8.5 L7 2 L9 8.5 L12.5 2 L12.5 12" stroke={color} strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
        </svg>
      );
    case "Gradle":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden style={{ flexShrink: 0 }}>
          <ellipse cx="6.5" cy="6" rx="4.5" ry="3.5" fill={color} opacity="0.75" />
          <rect x="3" y="8.5" width="2" height="3.5" rx="1" fill={color} opacity="0.75" />
          <rect x="8" y="8.5" width="2" height="3.5" rx="1" fill={color} opacity="0.75" />
          <path d="M11 5 Q14.5 3.5 13.5 7" stroke={color} strokeWidth="1.4" strokeLinecap="round" fill="none" />
        </svg>
      );
    case ".NET":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M7 1 L13 7 L7 13 L1 7 Z" fill={color} opacity="0.12" stroke={color} strokeWidth="1.1" />
          <path d="M7 4.2 L9.8 7 L7 9.8 L4.2 7 Z" fill={color} opacity="0.55" />
        </svg>
      );
    case "Swift":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M2 11.5 C5 3.5 12 2.5 12.5 5 C13 7.5 9 9 7 10 C10.5 9.5 11.5 11.5 10 12.5 C7.5 13.5 3.5 12.5 2 11.5 Z" fill={color} opacity="0.8" />
        </svg>
      );
    case "Dart":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M1.5 7.5 L4.5 1.5 L13 1.5 L13 7 L2 13.5 Z" fill={color} opacity="0.65" />
          <path d="M4.5 1.5 L13 1.5 L13 7 L4.5 1.5Z" fill={color} opacity="0.3" />
        </svg>
      );
    case "Elixir":
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <path d="M7 1.2 C10.5 4 12.5 7 12 9.5 C11.5 12 9.5 13.5 7 13.5 C4.5 13.5 2.5 12 2 9.5 C1.5 7 3.5 4 7 1.2 Z" fill={color} opacity="0.6" />
          <path d="M7 1.2 C6.5 5 5 7.5 5 9.5 C5 11 5.9 12.2 7 13" stroke={color} strokeWidth="1" opacity="0.45" />
        </svg>
      );
    default:
      return (
        <svg width="13" height="13" viewBox="0 0 14 14" fill="none" aria-hidden style={{ flexShrink: 0 }}>
          <circle cx="7" cy="7" r="5.5" stroke={color} strokeWidth="1.2" />
        </svg>
      );
  }
}

export const Clients = () => (
  <>
    <p className="mb-12 text-center text-lg text-white md:text-xl">
      <span className="text-primary-text">
        13 ecosystems, zero configuration.
      </span>
      <br className="hidden md:block" /> Auto-detected from your manifest files.
    </p>

    <div className="flex flex-wrap justify-center gap-x-3 gap-y-4">
      {ecosystems.map(({ name, color }) => (
        <span
          key={name}
          className="inline-flex items-center gap-[7px] rounded-full border px-4 py-[7px] text-sm font-medium"
          style={{
            color,
            borderColor: `${color}33`,
            backgroundColor: `${color}0d`,
          }}
        >
          <EcosystemIcon name={name} color={color} />
          {name}
        </span>
      ))}
    </div>
  </>
);
