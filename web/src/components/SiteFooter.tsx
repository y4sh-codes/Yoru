export const SiteFooter = () => {
  return (
    <footer className="mt-32 pb-10">
      <div className="mx-6 md:mx-10 my-10 px-8 py-20 flex flex-col items-center gap-6">
        <div className="font-display text-2xl flex items-center gap-2">
          Yoru
          <span className="text-[10px] font-mono px-1.5 py-0.5 bg-secondary rounded-sm text-muted-foreground">RS</span>
        </div>
        <div className="mt-12 flex items-center gap-8 text-caption">
          <a href="mailto:yashrajsingh231105@gmail.com" className="hover:text-foreground transition-colors">Contact</a>
          <a href="https://github.com/y4sh-codes/Yoru/blob/main/LICENSE" className="hover:text-foreground transition-colors">License · MIT</a>
          <a href="https://github.com/y4sh-codes/Yoru/releases" className="hover:text-foreground transition-colors">Changelog</a>
        </div>
      </div>
      <div className="text-center text-caption flex items-center justify-center gap-4">
        <span>© {new Date().getFullYear()} YORU. ALL RIGHTS RESERVED</span>
        <span className="opacity-40">·</span>
        <a href="https://github.com/y4sh-codes/Yoru" className="hover:text-foreground">GH</a>
      </div>
    </footer>
  );
};
