"use client";

import { cn } from "../../lib/utils";
import {
  CSSProperties,
  ReactElement,
  ReactNode,
  useEffect,
  useRef,
  useState,
} from "react";

interface NeonColorsProps {
  firstColor: string;
  secondColor: string;
  thirdColor: string;
}

interface NeonGradientCardProps {
  /**
   * @default <div />
   * @type ReactElement
   * @description
   * The component to be rendered as the card
   * */
  as?: ReactElement;
  /**
   * @default ""
   * @type string
   * @description
   * The className of the card
   */
  className?: string;

  /**
   * @default ""
   * @type ReactNode
   * @description
   * The children of the card
   * */
  children?: ReactNode;

  /**
   * @default 5
   * @type number
   * @description
   * The size of the border in pixels
   * */
  borderSize?: number;

  /**
   * @default 20
   * @type number
   * @description
   * The size of the radius in pixels
   * */
  borderRadius?: number;

  /**
   * @default "{ firstColor: '#ffaa40', secondColor: '#9c40ff' }"
   * @type string
   * @description
   * The colors of the neon gradient
   * */
  neonColors?: NeonColorsProps;

  height?: string;

  [key: string]: any;
}

const NeonGradientCard: React.FC<NeonGradientCardProps> = ({
  className,
  children,
  borderSize = 0,
  borderRadius = 20,
  height = '26rem',
  neonColors = {
    firstColor: "#ffaa40",
    secondColor: "#9c40ff",
    thirdColor: "#00FFF1",
  },
  ...props
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [dimensions, setDimensions] = useState({ width: 0, height: 0 });

  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        const { offsetWidth, offsetHeight } = containerRef.current;
        setDimensions({ width: offsetWidth, height: offsetHeight });
      }
    };

    updateDimensions();
    window.addEventListener("resize", updateDimensions);

    return () => {
      window.removeEventListener("resize", updateDimensions);
    };
  }, []);

  useEffect(() => {
    if (containerRef.current) {
      const { offsetWidth, offsetHeight } = containerRef.current;
      setDimensions({ width: offsetWidth, height: offsetHeight });
    }
  }, [children]);

  return (
    <div
      ref={containerRef}
      style={
        {
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
          // "--pseudo-element-height": `${dimensions.height + borderSize * 2}px`,
          "--pseudo-element-height": height,
          "--after-blur": `${dimensions.width / 3}px`,
        } as CSSProperties
      }
      className={cn(
        "relative z-10 h-full w-full rounded-[var(--border-radius)]",
        className,
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
          // "dark:bg-transparent dark:before:bg-transparent dark:after:bg-transparent",
          "dark:bg-[rgb(2,2,2)]",
          "navbar--fixed-top",
        )}
      >
        {children}
      </div>
    </div>
  );
};

export default NeonGradientCard;
