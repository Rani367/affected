import type { Metadata } from "next";
import { Footer } from "../components/footer";
import { Header } from "../components/header";
import "../styles/globals.css";

const SITE_URL = "https://rani367.github.io/affected";
const TITLE = "affected — Monorepo Affected Package CLI";
const DESCRIPTION =
  "A standalone CLI that detects affected packages in your monorepo. Zero config, 13 ecosystems, one binary.";

export const metadata: Metadata = {
  title: TITLE,
  description: DESCRIPTION,
  metadataBase: new URL(SITE_URL),
  alternates: {
    canonical: "/",
  },
  robots: {
    index: true,
    follow: true,
  },
  openGraph: {
    type: "website",
    title: TITLE,
    description: DESCRIPTION,
    url: `${SITE_URL}/`,
    siteName: "affected",
    images: [
      {
        url: `${SITE_URL}/og-image.png`,
        width: 1200,
        height: 630,
      },
    ],
    locale: "en_US",
  },
  twitter: {
    card: "summary_large_image",
    title: TITLE,
    description: DESCRIPTION,
    images: [`${SITE_URL}/og-image.png`],
  },
};

const jsonLd = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": `${SITE_URL}/#website`,
      name: "affected",
      url: `${SITE_URL}/`,
    },
    {
      "@type": "WebPage",
      "@id": `${SITE_URL}/#webpage`,
      name: TITLE,
      url: `${SITE_URL}/`,
      description: DESCRIPTION,
      inLanguage: "en",
      isPartOf: { "@id": `${SITE_URL}/#website` },
      mainEntity: { "@id": `${SITE_URL}/#software` },
    },
    {
      "@type": "SoftwareApplication",
      "@id": `${SITE_URL}/#software`,
      name: "affected",
      url: `${SITE_URL}/`,
      description: DESCRIPTION,
      applicationCategory: "DeveloperApplication",
      operatingSystem: "macOS, Linux, Windows",
      softwareVersion: "1.0.0",
      license: "https://opensource.org/licenses/MIT",
      offers: {
        "@type": "Offer",
        price: "0",
        priceCurrency: "USD",
      },
      downloadUrl: "https://github.com/Rani367/affected/releases",
      author: {
        "@type": "Person",
        name: "Rani367",
        url: "https://github.com/Rani367",
      },
    },
    {
      "@type": "SoftwareSourceCode",
      name: "affected",
      codeRepository: "https://github.com/Rani367/affected",
      programmingLanguage: "Rust",
      runtimePlatform: "Native binary",
      license: "https://opensource.org/licenses/MIT",
      author: {
        "@type": "Person",
        name: "Rani367",
        url: "https://github.com/Rani367",
      },
    },
    {
      "@type": "BreadcrumbList",
      itemListElement: [
        {
          "@type": "ListItem",
          position: 1,
          name: "affected",
          item: `${SITE_URL}/`,
        },
      ],
    },
  ],
};

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="en">
      <head>
        <meta name="theme-color" content="#0a0a0a" />
        <script
          type="application/ld+json"
          dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd) }}
        />
      </head>
      <body>
        <div>
          <Header />
          <main className="bg-page-gradient pt-navigation-height">
            {children}
          </main>
          <Footer />
        </div>
      </body>
    </html>
  );
}
