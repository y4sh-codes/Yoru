import { useEffect, useRef, useState } from "react";

/**
 * FragmentLetter — a giant letter rendered as a particle field on a canvas.
 * - Letter is fully visible (no slice clipping).
 * - On hover, particles scatter outward from the cursor and spring back
 *   (same effect used by the CTA buttons).
 * - On scroll-in, particles fly in from random offsets and assemble.
 */
type Particle = {
  x: number; y: number;
  ox: number; oy: number;
  vx: number; vy: number;
};

interface Props {
  letter: string;
  className?: string;
  delay?: number;
  /** Approximate pixel height of the rendered letter. */
  size?: number;
}

export const FragmentLetter = ({
  letter,
  className = "",
  delay = 0,
  size,
}: Props) => {
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rafRef = useRef<number | null>(null);
  const particlesRef = useRef<Particle[]>([]);
  const stateRef = useRef<{
    hover: boolean; mx: number; my: number;
    revealed: boolean; revealStart: number;
  }>({ hover: false, mx: -9999, my: -9999, revealed: false, revealStart: 0 });
  const [dims, setDims] = useState<{ w: number; h: number } | null>(null);

  // Measure container, then size the canvas to it.
  useEffect(() => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    const measure = () => {
      const w = wrap.clientWidth;
      // Letter height — provided, or scaled from container width
      const h = size ?? Math.min(w * 1.15, window.innerHeight * 0.55);
      setDims({ w: Math.max(80, Math.floor(w)), h: Math.max(80, Math.floor(h)) });
    };
    measure();
    const ro = new ResizeObserver(measure);
    ro.observe(wrap);
    window.addEventListener("resize", measure);
    return () => { ro.disconnect(); window.removeEventListener("resize", measure); };
  }, [size]);

  // Build particle field whenever dimensions change
  useEffect(() => {
    if (!dims) return;
    const { w: width, h: height } = dims;
    const canvas = canvasRef.current;
    if (!canvas) return;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    canvas.width = width * dpr;
    canvas.height = height * dpr;
    canvas.style.width = `${width}px`;
    canvas.style.height = `${height}px`;
    const ctx = canvas.getContext("2d")!;
    ctx.setTransform(1, 0, 0, 1, 0, 0);
    ctx.scale(dpr, dpr);

    // Off-screen rasteriser
    const off = document.createElement("canvas");
    off.width = width; off.height = height;
    const octx = off.getContext("2d")!;
    octx.fillStyle = "#000";
    octx.fillRect(0, 0, width, height);
    octx.fillStyle = "#fff";
    octx.textAlign = "center";
    octx.textBaseline = "middle";
    const fontSize = Math.floor(height * 0.95);
    octx.font = `800 ${fontSize}px Geist, Inter, system-ui, sans-serif`;
    octx.fillText(letter, width / 2, height / 2 + fontSize * 0.06);

    const data = octx.getImageData(0, 0, width, height).data;
    // Density: keep particle count manageable on big letters
    const targetMax = 4500;
    let step = 3;
    // Estimate count, bump step up if too many
    let est = 0;
    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width; x += step) {
        const i = (y * width + x) * 4;
        if (data[i] > 180) est++;
      }
    }
    while (est > targetMax && step < 8) {
      step++;
      est = 0;
      for (let y = 0; y < height; y += step) {
        for (let x = 0; x < width; x += step) {
          const i = (y * width + x) * 4;
          if (data[i] > 180) est++;
        }
      }
    }

    const particles: Particle[] = [];
    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width; x += step) {
        const i = (y * width + x) * 4;
        if (data[i] > 180) {
          // Start scattered around the canvas; will assemble on reveal
          const angle = Math.random() * Math.PI * 2;
          const r = Math.max(width, height) * (0.5 + Math.random() * 0.4);
          particles.push({
            x: width / 2 + Math.cos(angle) * r,
            y: height / 2 + Math.sin(angle) * r,
            ox: x, oy: y,
            vx: 0, vy: 0,
          });
        }
      }
    }
    particlesRef.current = particles;
  }, [dims, letter]);

  // Reveal trigger — IO + delay
  useEffect(() => {
    const wrap = wrapRef.current;
    if (!wrap) return;
    const io = new IntersectionObserver(
      (entries) => entries.forEach((e) => {
        if (e.isIntersecting) {
          window.setTimeout(() => {
            stateRef.current.revealed = true;
            stateRef.current.revealStart = performance.now();
          }, delay);
          io.disconnect();
        }
      }),
      { threshold: 0.15 }
    );
    io.observe(wrap);
    return () => io.disconnect();
  }, [delay]);

  // Animation loop
  useEffect(() => {
    if (!dims) return;
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);
    const { w: width, h: height } = dims;

    const tick = () => {
      const { hover, mx, my, revealed } = stateRef.current;

      ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);

      const ps = particlesRef.current;
      ctx.fillStyle = "hsl(0, 0%, 90%)";

      // While not revealed, particles drift slowly toward origin (assemble)
      const springK = revealed ? 0.075 : 0.025;
      const damp = revealed ? 0.8 : 0.9;

      for (let i = 0; i < ps.length; i++) {
        const p = ps[i];

        if (revealed && hover) {
          const dx = p.x - mx;
          const dy = p.y - my;
          const dist2 = dx * dx + dy * dy;
          const radius = Math.min(width, height) * 0.5;
          if (dist2 < radius * radius) {
            const dist = Math.sqrt(dist2) || 1;
            const force = (1 - dist / radius) * 9;
            p.vx += (dx / dist) * force + (Math.random() - 0.5) * 1.6;
            p.vy += (dy / dist) * force + (Math.random() - 0.5) * 1.6;
          }
        }

        const sx = (p.ox - p.x) * springK;
        const sy = (p.oy - p.y) * springK;
        p.vx = (p.vx + sx) * damp;
        p.vy = (p.vy + sy) * damp;
        p.x += p.vx;
        p.y += p.vy;

        ctx.fillRect(p.x, p.y, 1.6, 1.6);
      }

      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, [dims]);

  return (
    <div
      ref={wrapRef}
      onMouseEnter={() => { stateRef.current.hover = true; }}
      onMouseLeave={() => { stateRef.current.hover = false; }}
      onMouseMove={(e) => {
        const r = (e.currentTarget as HTMLDivElement).getBoundingClientRect();
        stateRef.current.mx = e.clientX - r.left;
        stateRef.current.my = e.clientY - r.top;
      }}
      className={`relative inline-block select-none ${className}`}
      aria-hidden
    >
      <canvas ref={canvasRef} className="block" />
    </div>
  );
};
