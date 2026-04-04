import Link from "next/link";
import { Container } from "./container";
import { GithubIcon } from "./icons/github";

const footerLinks = [
  {
    title: "Product",
    links: [
      { title: "Features", href: "#features" },
      { title: "Ecosystems", href: "#ecosystems" },
      { title: "Install", href: "#install" },
      { title: "Changelog", href: "https://github.com/Rani367/affected/releases" },
      { title: "GitHub", href: "https://github.com/Rani367/affected" },
    ],
  },
  {
    title: "Resources",
    links: [
      { title: "Documentation", href: "https://github.com/Rani367/affected#readme" },
      { title: "Issues", href: "https://github.com/Rani367/affected/issues" },
      { title: "Crates.io", href: "https://crates.io/crates/affected-cli" },
      { title: "MIT License", href: "https://github.com/Rani367/affected/blob/main/LICENSE" },
    ],
  },
];

export const Footer = () => (
  <footer className="mt-12 border-t border-transparent-white py-[5.6rem] text-sm">
    <Container className="flex flex-col justify-between lg:flex-row">
      <div>
        <div className="flex h-full flex-row justify-between lg:flex-col">
          <div className="flex items-center text-grey">
            <span className="mr-4 font-mono">affected</span> — Built with Rust
          </div>
          <div className="mt-auto flex space-x-4 text-grey">
            <Link href="https://github.com/Rani367/affected" aria-label="GitHub repository">
              <GithubIcon />
            </Link>
          </div>
        </div>
      </div>
      <div className="flex flex-wrap">
        {footerLinks.map((column) => (
          <div
            key={column.title}
            className="mt-10 min-w-[50%] lg:mt-0 lg:min-w-[18rem]"
          >
            <h3 className="mb-3 font-medium">{column.title}</h3>
            <ul>
              {column.links.map((link) => (
                <li key={link.title} className="[&_a]:last:mb-0">
                  <Link
                    className="mb-3 block text-grey transition-colors hover:text-off-white"
                    href={link.href}
                  >
                    {link.title}
                  </Link>
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </Container>
  </footer>
);
