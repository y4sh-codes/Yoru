import { useEffect, useRef } from "react";
import { Link } from "react-router-dom";
import { TopBar } from "@/components/TopBar";
import { SiteFooter } from "@/components/SiteFooter";
import { ScatterButton } from "@/components/ScatterButton";
import { FragmentLetter } from "@/components/FragmentLetter";

const Reveal = ({ children, delay = 0, className = "" }: { children: React.ReactNode; delay?: number; className?: string }) => {
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const el = ref.current;
    if (!el) return;
    const io = new IntersectionObserver(
      (entries) => entries.forEach((e) => {
        if (e.isIntersecting) {
          (e.target as HTMLElement).style.transitionDelay = `${delay}ms`;
          e.target.classList.add("in");
        }
      }),
      { threshold: 0.2 }
    );
    io.observe(el);
    return () => io.disconnect();
  }, [delay]);
  return <div ref={ref} className={`scroll-fade ${className}`}>{children}</div>;
};

const Index = () => {
  return (
    <div className="min-h-screen bg-background text-foreground overflow-x-hidden">
      <TopBar />

      {/* HERO ============================================================= */}
      <section className="relative pt-20 px-6 md:px-10">
        {/* Side metadata */}
        <div className="grid grid-cols-12 gap-6 mt-8">
          <div className="col-span-6 md:col-span-3 text-caption">
            v0.1.0 RELEASE<br/>BUILT WITH RUST
          </div>
          <div className="col-span-6 md:col-span-3 col-start-1 md:col-start-10 md:text-right text-caption">
            POSTMAN FOR<br/>THE SHELL
          </div>
        </div>

        {/* Big fragmented YORU letters — hover any letter to reveal the full glyph */}
        <div className="relative mt-10 md:mt-16 flex justify-between items-end gap-1 sm:gap-2 md:gap-6 min-h-[200px] sm:min-h-[320px] md:min-h-[520px] select-none">
          <FragmentLetter letter="Y" delay={0}   className="self-end" />
          <FragmentLetter letter="O" delay={120} className="self-start" />
          <FragmentLetter letter="R" delay={240} className="self-center" />
          <FragmentLetter letter="U" delay={360} className="self-end" />
        </div>

        {/* CTA + description */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 mt-20 mb-32">
          <div className="md:col-span-5">
            <ScatterButton label="Get Started" to="/docs#install" width={460} height={150} />
            <div className="mt-4 text-caption">
              ALREADY INSTALLED?{" "}
              <Link to="/docs" className="text-foreground underline-offset-4 hover:underline">READ DOCS</Link>
            </div>
          </div>
          <div className="md:col-span-5 md:col-start-7 text-sm leading-relaxed text-muted-foreground max-w-md font-mono">
            A FULL-FEATURED, KEYBOARD-DRIVEN HTTP API CLIENT THAT LIVES ENTIRELY IN
            YOUR TERMINAL. COLLECTIONS, ENVIRONMENTS, AUTH, SCRIPTING — FOR
            DEVELOPERS WHO LIVE IN THE SHELL.
          </div>
        </div>
      </section>

      {/* MARQUEE STRIP ==================================================== */}
      <section className="py-4 overflow-hidden">
        <div className="flex animate-marquee whitespace-nowrap text-caption">
          {Array.from({ length: 2 }).map((_, k) => (
            <div key={k} className="flex shrink-0 gap-12 pr-12">
              {["RATATUI", "CROSSTERM", "RHAI SCRIPTING", "ATOMIC SAVES", "YAML / JSON", "BEARER · BASIC · API KEY", "ENVIRONMENT VARS", "REQUEST HISTORY", "FUZZY FILTER", "CLI ONE-SHOT"].map((w) => (
                <span key={w}>◆ {w}</span>
              ))}
            </div>
          ))}
        </div>
      </section>

      {/* FEATURES — fragmented letters interleaved with copy ============= */}
      <section className="px-6 md:px-10 mt-32">
        <div className="text-caption mb-16">[ FEATURES ]  ·  SCROLL TO REVEAL</div>

        {/* Row 1 */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 items-center mb-32">
          <div className="md:col-span-5">
            <FragmentLetter letter="Y" />
          </div>
          <div className="md:col-span-3 md:col-start-7">
            <Reveal>
              <div className="text-caption mb-3">01 · INTERFACE</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                INTERACTIVE TUI BUILT WITH RATATUI + CROSSTERM. RENDERS IN ANY MODERN
                TERMINAL WITH 24-BIT COLOUR.
              </p>
            </Reveal>
          </div>
          <div className="md:col-span-3 md:col-start-10">
            <Reveal delay={120}>
              <div className="text-caption mb-3">02 · COLLECTIONS</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                ORGANISE REQUESTS INTO NAMED COLLECTIONS. CREATE, RENAME, AND DUPLICATE
                ON THE FLY.
              </p>
            </Reveal>
          </div>
        </div>

        {/* Row 2 */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 items-center mb-32">
          <div className="md:col-span-3">
            <Reveal>
              <div className="text-caption mb-3">03 · METHOD BADGES</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                COLOUR-CODED GET / POST / PUT / PATCH / DELETE LABELS IN THE NAVIGATOR.
              </p>
            </Reveal>
          </div>
          <div className="md:col-span-5 md:col-start-5">
            <FragmentLetter letter="O" />
          </div>
          <div className="md:col-span-3 md:col-start-10">
            <Reveal delay={120}>
              <div className="text-caption mb-3">04 · AUTH</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                NONE · BEARER TOKEN · HTTP BASIC · API KEY (HEADER OR QUERY). ALL
                CONFIGURED INLINE.
              </p>
            </Reveal>
          </div>
        </div>

        {/* Row 3 */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 items-center mb-32">
          <div className="md:col-span-5">
            <FragmentLetter letter="R" />
          </div>
          <div className="md:col-span-3 md:col-start-7">
            <Reveal>
              <div className="text-caption mb-3">05 · ENVIRONMENTS</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                NAMED ENVIRONMENTS WITH {"{{var}}"} INTERPOLATION. CYCLE WITH `e`.
              </p>
            </Reveal>
          </div>
          <div className="md:col-span-3 md:col-start-10">
            <Reveal delay={120}>
              <div className="text-caption mb-3">06 · SCRIPTS</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                INLINE PRE-REQUEST AND TEST SCRIPTS POWERED BY RHAI. LOG TO THE LOGS TAB.
              </p>
            </Reveal>
          </div>
        </div>

        {/* Row 4 */}
        <div className="grid grid-cols-1 md:grid-cols-12 gap-10 items-center mb-32">
          <div className="md:col-span-3">
            <Reveal>
              <div className="text-caption mb-3">07 · HISTORY</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                LAST 500 EXECUTIONS WITH LATENCY, SIZE AND STATUS — FILTERABLE PER
                COLLECTION.
              </p>
            </Reveal>
          </div>
          <div className="md:col-span-5 md:col-start-5">
            <FragmentLetter letter="U" />
          </div>
          <div className="md:col-span-3 md:col-start-10">
            <Reveal delay={120}>
              <div className="text-caption mb-3">08 · CLI ONE-SHOT</div>
              <p className="text-sm text-foreground/90 leading-relaxed font-mono">
                <code className="text-foreground">yoru send</code> FOR SCRIPTS, CRON
                JOBS AND CI PIPELINES. NO TUI REQUIRED.
              </p>
            </Reveal>
          </div>
        </div>
      </section>

      {/* TERMINAL DEMO ==================================================== */}
      <section className="px-6 md:px-10 mt-20">
        <div className="text-caption mb-6">[ PREVIEW ]  ·  IN THE TERMINAL</div>
        <Reveal>
          <div className="rounded-md bg-card overflow-hidden shadow-2xl">
            <div className="flex items-center gap-2 px-4 py-2.5 bg-secondary/30">
              <span className="w-3 h-3 rounded-full bg-[#ff5f57]" />
              <span className="w-3 h-3 rounded-full bg-[#febc2e]" />
              <span className="w-3 h-3 rounded-full bg-[#28c840]" />
              <span className="ml-3 text-caption">~/projects/api ─ yoru</span>
            </div>
            <div className="grid grid-cols-12 text-[12.5px] font-mono">
              <div className="col-span-3 p-4 space-y-2">
                <div className="text-caption mb-3">COLLECTIONS</div>
                <div className="text-muted-foreground">▾ Auth API</div>
                <div className="pl-3 flex items-center gap-2"><span className="text-[10px] px-1.5 py-0.5 bg-emerald-500/15 text-emerald-400 rounded-sm">GET</span> <span className="text-foreground">whoami</span></div>
                <div className="pl-3 flex items-center gap-2"><span className="text-[10px] px-1.5 py-0.5 bg-amber-500/15 text-amber-400 rounded-sm">POST</span> <span className="text-muted-foreground">login</span></div>
                <div className="pl-3 flex items-center gap-2"><span className="text-[10px] px-1.5 py-0.5 bg-sky-500/15 text-sky-400 rounded-sm">PUT</span> <span className="text-muted-foreground">profile</span></div>
                <div className="pl-3 flex items-center gap-2"><span className="text-[10px] px-1.5 py-0.5 bg-rose-500/15 text-rose-400 rounded-sm">DEL</span> <span className="text-muted-foreground">session</span></div>
                <div className="text-muted-foreground mt-3">▸ Users</div>
                <div className="text-muted-foreground">▸ Billing</div>
              </div>
              <div className="col-span-5 p-4 space-y-2">
                <div className="text-caption mb-3">REQUEST</div>
                <div className="flex items-center gap-2">
                  <span className="text-[10px] px-1.5 py-0.5 bg-emerald-500/15 text-emerald-400 rounded-sm">GET</span>
                  <span className="text-foreground truncate">{"{{base_url}}/auth/whoami"}</span>
                </div>
                <div className="text-caption mt-4">HEADERS</div>
                <div className="text-muted-foreground">Accept: application/json</div>
                <div className="text-muted-foreground">Authorization: Bearer {"{{token}}"}</div>
                <div className="text-caption mt-4">BODY · NONE</div>
                <div className="text-caption mt-4">SCRIPT</div>
                <pre className="text-muted-foreground whitespace-pre-wrap">{`if vars["env"] == "prod" {
  log("⚠ Targeting production!");
}`}</pre>
              </div>
              <div className="col-span-4 p-4 space-y-2">
                <div className="text-caption mb-3">RESPONSE · 200 OK · 142 ms</div>
                <pre className="text-foreground/90 whitespace-pre overflow-hidden">{`{
  "id": "u_8a91",
  "email": "dev@yoru.sh",
  "role": "admin",
  "ok": true
}`}</pre>
                <div className="text-caption mt-6">LOGS</div>
                <div className="text-muted-foreground">› GET /auth/whoami → 200</div>
                <div className="text-muted-foreground">› 142 ms · 84 B</div>
              </div>
              <div className="col-span-12 px-4 py-2 bg-secondary/40 text-caption flex justify-between">
                <span>[r] RUN  [m] METHOD  [u] URL  [h] HEADER  [/] FILTER  [?] HELP</span>
                <span className="text-foreground">▌</span>
              </div>
            </div>
          </div>
        </Reveal>
      </section>

      {/* CTA STRIP ======================================================== */}
      <section className="px-6 md:px-10 mt-32">
        <div className="grid grid-cols-1 md:grid-cols-12 gap-6">
          <div className="md:col-span-6 text-caption">JOIN THE YORU<br/>COMMUNITY</div>
          <div className="md:col-span-6 md:text-right text-caption">v0.1 · MIT LICENSE<br/>RUST 1.85+</div>
        </div>
        <div className="mt-10 px-4 py-20 md:py-32 flex items-center justify-center">
          <ScatterButton label="Install Yoru" to="/docs#install" width={520} height={150} />
        </div>
      </section>

      <SiteFooter />
    </div>
  );
};

export default Index;
