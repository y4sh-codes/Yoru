import { useEffect, useRef, useState } from "react";
import { Link } from "react-router-dom";

/**
 * ScatterButton — a giant fragmented-style label rendered as a particle field
 * on a transparent canvas, sitting on a BLACK plate (matching the YORU
 * typography). Hovering the cursor disperses nearby particles in a spray /
 * smoke effect; they spring back to form the word again.
 */
type Particle = {
  x: number; y: number;
  ox: number; oy: number;
  vx: number; vy: number;
};

interface Props {
  label: string;
  to: string;
  width?: number;
  height?: number;
}

export const ScatterButton = ({ label, to, width = 560, height = 160 }: Props) => {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const wrapRef = useRef<HTMLAnchorElement | null>(null);
  const rafRef = useRef<number | null>(null);
  const particlesRef = useRef<Particle[]>([]);
  const stateRef = useRef<{ hover: boolean; mx: number; my: number }>({
    hover: false, mx: -9999, my: -9999,
  });
  const [ready, setReady] = useState(false);

  // Build particle field from the rasterised label
  useEffect(() => {
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

    // Off-screen rasteriser (white text on black, used as a mask)
    const off = document.createElement("canvas");
    off.width = width; off.height = height;
    const octx = off.getContext("2d")!;
    octx.fillStyle = "#000";
    octx.fillRect(0, 0, width, height);
    octx.fillStyle = "#fff";
    octx.textAlign = "center";
    octx.textBaseline = "middle";
    // Auto-fit font size so descenders/ascenders never clip
    let fontSize = Math.floor(height * 0.55);
    octx.font = `800 ${fontSize}px Geist, Inter, system-ui, sans-serif`;
    const maxWidth = width * 0.86;
    let measured = octx.measureText(label).width;
    if (measured > maxWidth) {
      fontSize = Math.floor(fontSize * (maxWidth / measured));
      octx.font = `800 ${fontSize}px Geist, Inter, system-ui, sans-serif`;
    }
    octx.fillText(label, width / 2, height / 2);

    const data = octx.getImageData(0, 0, width, height).data;
    const step = 3;
    const particles: Particle[] = [];
    for (let y = 0; y < height; y += step) {
      for (let x = 0; x < width; x += step) {
        const i = (y * width + x) * 4;
        if (data[i] > 180) {
          particles.push({ x, y, ox: x, oy: y, vx: 0, vy: 0 });
        }
      }
    }
    particlesRef.current = particles;
    setReady(true);

    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, [label, width, height]);

  // Animation loop
  useEffect(() => {
    if (!ready) return;
    const canvas = canvasRef.current!;
    const ctx = canvas.getContext("2d")!;
    const dpr = Math.min(window.devicePixelRatio || 1, 2);

    const tick = () => {
      const { hover, mx, my } = stateRef.current;

      // Clear (transparent canvas — the wrapper provides the black plate)
      ctx.clearRect(0, 0, canvas.width / dpr, canvas.height / dpr);

      const ps = particlesRef.current;
      // Off-white particles — same tone as the YORU letters
      ctx.fillStyle = "hsl(0, 0%, 90%)";

      for (let i = 0; i < ps.length; i++) {
        const p = ps[i];

        if (hover) {
          const dx = p.x - mx;
          const dy = p.y - my;
          const dist2 = dx * dx + dy * dy;
          const radius = 140;
          if (dist2 < radius * radius) {
            const dist = Math.sqrt(dist2) || 1;
            const force = (1 - dist / radius) * 9;
            p.vx += (dx / dist) * force + (Math.random() - 0.5) * 1.6;
            p.vy += (dy / dist) * force + (Math.random() - 0.5) * 1.6;
          }
        }

        // spring back to origin
        const sx = (p.ox - p.x) * 0.075;
        const sy = (p.oy - p.y) * 0.075;
        p.vx = (p.vx + sx) * 0.8;
        p.vy = (p.vy + sy) * 0.8;
        p.x += p.vx;
        p.y += p.vy;

        ctx.fillRect(p.x, p.y, 1.6, 1.6);
      }

      rafRef.current = requestAnimationFrame(tick);
    };
    rafRef.current = requestAnimationFrame(tick);
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current); };
  }, [ready, width, height]);

  return (
    <Link
      ref={wrapRef}
      to={to}
      onMouseEnter={() => { stateRef.current.hover = true; }}
      onMouseLeave={() => { stateRef.current.hover = false; }}
      onMouseMove={(e) => {
        const r = (e.currentTarget as HTMLAnchorElement).getBoundingClientRect();
        stateRef.current.mx = e.clientX - r.left;
        stateRef.current.my = e.clientY - r.top;
      }}
      className="relative inline-block group rounded-sm overflow-hidden bg-background transition-opacity hover:opacity-95"
      aria-label={label}
      style={{ width, height, maxWidth: "100%" }}
    >
      {/* Subtle inner vignette for depth */}
      <span
        aria-hidden
        className="absolute inset-0 pointer-events-none"
        style={{
          background:
            "radial-gradient(ellipse at center, transparent 55%, hsl(0 0% 0% / 0.6) 100%)",
        }}
      />
      <canvas ref={canvasRef} className="block relative z-10" />
      <span className="sr-only">{label}</span>
    </Link>
  );
};
