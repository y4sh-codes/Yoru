import { Link, useLocation } from "react-router-dom";

export const TopBar = () => {
  const { pathname } = useLocation();
  return (
    <header className="fixed top-0 inset-x-0 z-50 bg-background/80 backdrop-blur-sm hairline-b">
      <div className="grid grid-cols-3 items-center px-6 md:px-10 h-14 font-mono">
        <div className="text-caption">v0.1 · TERMINAL · API CLIENT</div>
        <Link to="/" className="justify-self-center flex items-center gap-2 font-display text-base text-foreground">
          <span>Yoru</span>
          <span className="text-[10px] font-mono px-1.5 py-0.5 bg-secondary rounded-sm text-muted-foreground border border-border">RS</span>
        </Link>
        <nav className="justify-self-end flex items-center gap-5 text-[12px] uppercase tracking-wider">
          <Link
            to="/docs"
            className={`hover:text-foreground transition-colors ${pathname === "/docs" ? "text-foreground" : "text-muted-foreground"}`}
          >
            Docs
          </Link>
          <a
            href="https://github.com/y4sh-codes/Yoru"
            target="_blank"
            rel="noreferrer"
            className="text-muted-foreground hover:text-foreground transition-colors"
          >
            GitHub
          </a>
          <Link
            to="/docs#install"
            className="inline-flex items-center gap-2 px-3 py-1.5 border border-border bg-secondary/40 hover:bg-secondary text-foreground rounded-sm"
          >
            Install <span aria-hidden>→</span>
          </Link>
        </nav>
      </div>
    </header>
  );
};
