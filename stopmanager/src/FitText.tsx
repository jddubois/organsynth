import { useLayoutEffect, useRef } from "react";

/** Text centered in its box, shrunk until it fits (never truncated). */
export function FitText({ text, max = 22, min = 10 }: { text: string; max?: number; min?: number }) {
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
  }, [text, max, min]);

  return (
    <div ref={ref} className="w-full h-full flex items-center justify-center overflow-hidden leading-tight text-center">
      {text}
    </div>
  );
}
