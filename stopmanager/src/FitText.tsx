import { useLayoutEffect, useRef } from "react";

/** Lines of text centered in their box, shrunk until they fit it (never truncated). */
export function FitText({ lines, max = 17, min = 9 }: { lines: string[]; max?: number; min?: number }) {
  const ref = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    const fit = () => {
      let size = max;
      el.style.fontSize = `${size}px`;
      while (size > min && (el.scrollHeight > el.clientHeight || el.scrollWidth > el.clientWidth)) {
        size -= 0.5;
        el.style.fontSize = `${size}px`;
      }
    };
    fit();
    const observer = new ResizeObserver(fit);
    observer.observe(el);
    return () => observer.disconnect();
  }, [lines.join("\n"), max, min]);

  return (
    <div
      ref={ref}
      className="w-full h-full flex flex-col items-center justify-center gap-[0.3em] overflow-hidden leading-tight text-center"
    >
      {lines.map((line) => (
        <span key={line}>{line}</span>
      ))}
    </div>
  );
}
