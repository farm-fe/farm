"use client";

import { cn } from "../../lib/utils";
import React, { CSSProperties, ReactNode, useEffect, useRef, useState, useMemo } from "react";

interface NeonColorsProps {
  firstColor: string;
  secondColor: string;
  thirdColor: string;
}

interface NeonGradientCardProps {
  className?: string;
  children?: ReactNode;
  borderSize?: number;
  borderRadius?: number;
  neonColors?: NeonColorsProps;
  height?: string;
  [key: string]: any;
}

const DEFAULT_NEON_COLORS: NeonColorsProps = {
  firstColor: "#ffaa40",
  secondColor: "#9c40ff",
  thirdColor: "#00FFF1",
};

const NeonGradientCard: React.FC<NeonGradientCardProps> = React.memo(({
  className,
  children,
  borderSize = 0,
  borderRadius = 20,
  height = '26rem',
  neonColors = DEFAULT_NEON_COLORS,
  ...props
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });
  const childrenRef = useRef(children);

  const updateDimensions = () => {
    if (containerRef.current) {
      const { offsetWidth, offsetHeight } = containerRef.current;
      setDimensions({ width: offsetWidth, height: offsetHeight });
    }
  };

  useEffect(() => {
    updateDimensions();
    window.addEventListener("resize", updateDimensions);
    return () => window.removeEventListener("resize", updateDimensions);
  }, []);

  useEffect(() => {
    if (childrenRef.current !== children) {
      childrenRef.current = children;
      updateDimensions();
    }
  }, [children]);

  const cardStyle = useMemo(() => {
    return {
      "--border-size": `${borderSize}px`,
      "--border-radius": `${borderRadius}px`,
      "--neon-first-color": neonColors.firstColor,
      "--neon-second-color": neonColors.secondColor,
      "--neon-third-color": neonColors.thirdColor,
      "--card-width": `${dimensions.width}px`,
      "--card-height": `${dimensions.height}px`,
      "--card-content-radius": `${borderRadius - borderSize}px`,
      "--pseudo-element-background-image": `linear-gradient(0deg, ${neonColors.firstColor}, ${neonColors.secondColor}, ${neonColors.thirdColor})`,
      "--pseudo-element-width": `${dimensions.width + borderSize * 2}px`,
      "--pseudo-element-height": height,
      "--after-blur": `${dimensions.width / 3}px`,
    } as CSSProperties;
  }, [borderSize, borderRadius, neonColors, dimensions, height]);

  return (
    <div
      ref={containerRef}
      style={cardStyle}
      className={cn(
        "relative z-10 h-full w-full rounded-[var(--border-radius)]",
        className
      )}
      {...props}
    >
      <div
        className={cn(
          "relative h-full min-h-[inherit] w-full rounded-[var(--card-content-radius)] bg-gray-100",
          "before:absolute before:top-[-var(--border-size)] before:left-0 before:right-0 before:z-[-10] before:block",
          "before:h-[var(--pseudo-element-height)] before:w-full before:content-['']",
          "before:bg-[var(--pseudo-element-background-image)] before:bg-[length:200%_100%]",
          "before:animate-backgroundPositionSpin",
          "after:absolute after:-left-[var(--border-size)] after:-top-[var(--border-size)] after:-z-10 after:block",
          "after:h-[var(--pseudo-element-height)] after:w-[var(--pseudo-element-width)] after:rounded-[var(--border-radius)] after:blur-[var(--after-blur)] after:content-['']",
          "after:bg-[linear-gradient(0deg,var(--neon-first-color),var(--neon-second-color),var(--neon-third-color))] after:bg-[length:100%_200%] after:opacity-80",
          "after:animate-backgroundPositionSpin",
          "after:animation-duration: 10s",
          "dark:bg-[rgb(2,2,2)]",
          "navbar--fixed-top"
        )}
      >
        {children}
      </div>
    </div>
  );
});

export default NeonGradientCard;
