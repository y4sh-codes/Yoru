import { useEffect, useState } from "react";
import { TopBar } from "@/components/TopBar";
import { SiteFooter } from "@/components/SiteFooter";

const sections = [
  { id: "overview", label: "Overview" },
  { id: "install", label: "Install" },
  { id: "running", label: "Building & Running" },
  { id: "tui", label: "TUI Keybindings" },
  { id: "cli", label: "CLI Commands" },
  { id: "scripts", label: "Inline Scripts" },
  { id: "workspace", label: "Workspace Format" },
  { id: "architecture", label: "Architecture" },
  { id: "roadmap", label: "Roadmap" },
];

const KeyTable = ({ rows }: { rows: [string, string][] }) => (
  <div className="hairline-top">
    {rows.map(([k, v]) => (
      <div key={k} className="grid grid-cols-12 gap-4 py-2.5 hairline-b text-sm">
        <code className="col-span-3 text-accent font-mono">{k}</code>
        <span className="col-span-9 text-foreground/85">{v}</span>
      </div>
    ))}
  </div>
);

const Code = ({ children }: { children: React.ReactNode }) => (
  <pre className="bg-card border border-border rounded-sm p-4 my-4 overflow-x-auto text-[12.5px] leading-relaxed font-mono text-foreground/90">{children}</pre>
);

const H = ({ id, children }: { id: string; children: React.ReactNode }) => (
  <h2 id={id} className="font-display text-3xl md:text-4xl mt-20 mb-6 scroll-mt-24">{children}</h2>
);

const Docs = () => {
  const [active, setActive] = useState("overview");

  useEffect(() => {
    const onScroll = () => {
      let current = "overview";
      for (const s of sections) {
        const el = document.getElementById(s.id);
        if (el && el.getBoundingClientRect().top < 120) current = s.id;
      }
      setActive(current);
    };
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <div className="min-h-screen bg-background text-foreground">
      <TopBar />
      <div className="pt-20 px-6 md:px-10 grid grid-cols-1 md:grid-cols-12 gap-10">
        {/* Sidebar */}
        <aside className="md:col-span-3 md:sticky md:top-20 md:self-start">
          <div className="text-caption mb-4">[ DOCUMENTATION ]</div>
          <h1 className="font-display text-4xl mb-2">Yoru Docs</h1>
          <p className="text-muted-foreground text-sm mb-8 font-mono">
            Everything you need to run Yoru — from install to Rhai scripting.
          </p>
          <nav className="space-y-1.5 text-sm font-mono">
            {sections.map((s) => (
              <a
                key={s.id}
                href={`#${s.id}`}
                className={`block py-1 transition-colors ${active === s.id ? "text-foreground" : "text-muted-foreground hover:text-foreground"}`}
              >
                <span className="opacity-60 mr-2">{active === s.id ? "▸" : "·"}</span>{s.label}
              </a>
            ))}
          </nav>
        </aside>

        {/* Content */}
        <article className="md:col-span-9 max-w-3xl">
          <H id="overview">Overview</H>
          <p className="text-foreground/90 leading-relaxed">
            Yoru is a full-featured, keyboard-driven HTTP API client that lives entirely
            in your terminal. It brings the core power of Postman — collections,
            environments, auth, body editing, history, scripting — to developers who
            live in the shell.
          </p>
          <div className="mt-6 grid grid-cols-2 md:grid-cols-3 gap-px bg-border border border-border rounded-sm overflow-hidden">
            {[
              ["Built with", "Rust + ratatui"],
              ["License", "MIT"],
              ["Min. Rust", "1.85"],
              ["Body types", "Raw · JSON · Form"],
              ["Auth", "Bearer · Basic · Key"],
              ["Storage", "YAML / JSON"],
            ].map(([k, v]) => (
              <div key={k} className="bg-background p-4">
                <div className="text-caption mb-1">{k}</div>
                <div className="text-sm text-foreground">{v}</div>
              </div>
            ))}
          </div>

          <H id="install">Install</H>
          <p className="text-foreground/90 leading-relaxed">
            Requires Rust ≥ 1.85 and a terminal with ANSI / 24-bit colour support
            (Kitty, Alacritty, iTerm2, Windows Terminal, etc.).
          </p>
          <p className="text-foreground/90 leading-relaxed mt-4">
            On Linux and macOS you can use the install script bundled with the
            repository. It clones the repo, builds the release binary with
            <code className="text-accent"> cargo</code>, and installs it to{" "}
            <code className="text-accent">~/.local/bin/yoru</code>.
          </p>
          <Code>{`# Linux / macOS — one-line install
curl -fsSL https://raw.githubusercontent.com/y4sh-codes/Yoru/main/install.sh | bash

# Make sure ~/.local/bin is on your PATH
export PATH="$HOME/.local/bin:$PATH"

# Verify
yoru --version`}</Code>
          <p className="text-foreground/90 leading-relaxed mt-4">Or build from source:</p>
          <Code>{`git clone https://github.com/y4sh-codes/Yoru
cd Yoru
cargo build --release
./target/release/yoru`}</Code>

          <H id="running">Building & Running</H>
          <Code>{`cargo build --release   # build release binary
cargo run               # launch the TUI
./target/release/yoru   # run the built binary`}</Code>

          <H id="tui">TUI Keybindings</H>

          <h3 className="text-caption mt-8 mb-3">Navigation</h3>
          <KeyTable rows={[
            ["↑ / ↓", "Move between requests"],
            ["← / →", "Switch collection"],
            ["/", "Open live request filter"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Requests</h3>
          <KeyTable rows={[
            ["r / Enter", "Run selected request"],
            ["n", "Add quick request to current collection"],
            ["d", "Duplicate selected request"],
            ["x", "Delete selected request"],
            ["m", "Cycle HTTP method GET→POST→PUT→PATCH→DELETE→HEAD→OPTIONS"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Editing</h3>
          <KeyTable rows={[
            ["i", "Edit request name"],
            ["u", "Edit URL"],
            ["h", "Add header — Key:Value"],
            ["p", "Add query parameter — key=value"],
            ["b", "Edit raw request body"],
            ["T", "Set timeout in ms (empty = default)"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Authentication</h3>
          <KeyTable rows={[
            ["t", "Set bearer token (empty clears auth)"],
            ["a", "Set Basic auth — username:password"],
            ["k", "Set API key — name:value or name:value:h / :q"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Collections</h3>
          <KeyTable rows={[
            ["N", "Create new collection"],
            ["C", "Rename current collection"],
            ["e", "Cycle active environment"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Response</h3>
          <KeyTable rows={[
            ["1 / 2 / 3 / 4", "Switch to Body / Headers / Logs / History tab"],
            ["Tab", "Cycle response tabs"],
            ["PgDn / PgUp", "Scroll response body"],
          ]} />

          <h3 className="text-caption mt-8 mb-3">Other</h3>
          <KeyTable rows={[
            ["?", "Toggle help overlay (also closed by Esc)"],
            ["c / Esc", "Clear error / close overlays"],
            ["q", "Quit"],
          ]} />

          <H id="cli">CLI Commands</H>
          <h3 className="text-caption mt-6 mb-2">One-shot request</h3>
          <Code>{`# Simple GET
yoru send --method GET --url "https://httpbin.org/get"

# POST with JSON body and bearer auth
yoru send \\
  --method POST \\
  --url "https://api.example.com/users" \\
  --json '{"name":"Alice","role":"admin"}' \\
  --bearer "YOUR_TOKEN_HERE"

# Environment variable interpolation
yoru send \\
  --method GET \\
  --url "https://httpbin.org/get?svc={{service}}" \\
  --env service=yoru \\
  --header "Accept:application/json"

# Basic auth
yoru send --method GET --url "https://httpbin.org/basic-auth/user/pass" \\
  --basic-user user --basic-password pass

# API key in header
yoru send --method GET --url "https://api.example.com/data" \\
  --api-key "X-API-Key=abc123"`}</Code>

          <h3 className="text-caption mt-6 mb-2">Workspace management</h3>
          <Code>{`yoru init --name "My Project"
yoru export --file ./backup/workspace.json
yoru export --file ./backup/workspace.yaml
yoru import --file ./backup/workspace.json`}</Code>

          <H id="scripts">Inline Scripts (Rhai)</H>
          <p className="text-foreground/90 leading-relaxed">
            Each request supports optional <code className="text-accent">pre_request_script</code> and{" "}
            <code className="text-accent">test_script</code> fields in the workspace YAML.
          </p>
          <h3 className="text-caption mt-6 mb-2">Available context</h3>
          <ul className="list-disc pl-6 space-y-1 text-foreground/90">
            <li><code className="text-accent">vars</code> — map of all environment + request variables</li>
            <li><code className="text-accent">log("message")</code> — emits a line to the Logs tab</li>
            <li><code className="text-accent">status</code> — HTTP status code (test script only)</li>
            <li><code className="text-accent">response_body</code> — raw response body (test script only)</li>
          </ul>
          <Code>{`// pre_request_script
if vars["env"] == "prod" {
    log("⚠ Targeting production!");
}

// test_script
if status != "200" {
    log("Unexpected status: " + status);
}`}</Code>

          <H id="workspace">Workspace File Format</H>
          <p className="text-foreground/90 leading-relaxed">
            Yoru stores everything in a single YAML (or JSON) workspace file:
          </p>
          <Code>{`~/.local/share/yoru/workspace.yaml   # Linux / macOS
%APPDATA%\\yoru\\workspace.yaml        # Windows`}</Code>
          <Code>{`name: My Workspace
collections:
  - name: Auth API
    requests:
      - name: Login
        method: POST
        url: "{{base_url}}/auth/login"
        body:
          kind: json
          value:
            email: "{{user_email}}"
            password: "{{user_password}}"
        auth:
          kind: none
environments:
  - name: local
    variables:
      - key: base_url
        value: http://localhost:3000
      - key: user_email
        value: dev@example.com`}</Code>

          <H id="architecture">Architecture</H>
          <Code>{`src/
├── core/      Domain models & validation
├── storage/   Persistence + filesystem backend (atomic writes)
├── http/      Executor, auth, templating, Rhai scripting
├── app/       State machine + stateful actions
├── tui/       Ratatui rendering, theme, event loop
│   ├── theme.rs
│   ├── ui.rs
│   ├── mod.rs
│   └── events.rs
├── cli/       Clap argument parsing & send command
└── util/      Error types, logging, time helpers`}</Code>

          <H id="roadmap">Roadmap</H>
          <ul className="space-y-2 text-foreground/90">
            {[
              "OAuth 2 token flows (client credentials, auth code)",
              "GraphQL request type with query/variables editor",
              "gRPC request type",
              "Multi-select request runner (run all in collection)",
              "Test suite reporting with pass/fail counts",
              "Response body syntax highlighting",
              "Team sync + encrypted secret store",
              "TUI environment variable editor",
              "Import from Postman / Insomnia collection JSON",
            ].map((r) => (
              <li key={r} className="flex gap-3 items-start">
                <span className="text-caption mt-1.5">▢</span>
                <span>{r}</span>
              </li>
            ))}
          </ul>
        </article>
      </div>
      <SiteFooter />
    </div>
  );
};

export default Docs;
