import { animated, useSpring } from "@react-spring/web";
import React, { useEffect, useState } from "react";
import styles from "./index.module.css";

export function formatTime(time: number, totalTime: number) {
  if (totalTime < 1000) {
    return `${time.toFixed(0)}ms`;
  } else {
    return `${(time / 1000).toFixed(2)}s`;
  }
}

export function ProgressBar({ value, max, animate = false, contained = false }) {
  const TOTAL_TIME = value * 1000;
  const [elapsedTime, setElapsedTime] = useState(TOTAL_TIME);
  const [isMobile, setIsMobile] = useState(false);
  useEffect(() => {
    const update = () => setIsMobile(window.innerWidth < 768);
    update();
    window.addEventListener("resize", update);
    return () => window.removeEventListener("resize", update);
  }, []);
  const progressBarWidth = isMobile ? 80 : 18;
  const bounded = contained || isMobile;
  const formattedTime = formatTime(elapsedTime, TOTAL_TIME);
  const [props, api] = useSpring(() => ({ width: "100%" }));
  useEffect(() => {
    if (!animate) {
      api.set({ width: "100%" });
      setElapsedTime(TOTAL_TIME);
      return;
    }
    setElapsedTime(0);
    api.start({
      from: { width: "0%" },
      to: { width: "100%" },
      reset: true,
      config: { duration: TOTAL_TIME },
      onChange(data) {
        setElapsedTime((parseFloat(data.value.width) / 100) * TOTAL_TIME);
      },
    });
    return () => { api.stop(); };
  }, [api, animate, TOTAL_TIME]);

  return (
    <div
      className={`${styles["progress-bar-container"]} flex justify-between items-center sm:pr-4`}
      style={{
        width: bounded ? "100%" : `${progressBarWidth}vw`,
        flex: 1,
        boxSizing: bounded ? "border-box" : undefined,
        maxWidth: bounded ? "100%" : undefined,
        minWidth: bounded ? 0 : undefined,
      }}
    >
      <div
        className={`${styles["progress-bar-inner-container"]} flex justify-between`}
        style={{
          width: bounded ? `${(value / max) * 80}%` : `${(value / max) * 0.8 * progressBarWidth}vw`,
        }}
      >
        <animated.div className={styles["progress-bar"]} style={props} />
      </div>
      <div className={`${styles["font-mono"]} text-sm sm:text-base`}>
        {formattedTime}
      </div>
    </div>
  );
}
