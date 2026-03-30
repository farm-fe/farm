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

export function ProgressBar({ value, max }) {
  const [elapsedTime, setElapsedTime] = useState(0);
  const TOTAL_TIME = value * 1000;
  const isMobile = window.innerWidth < 768;
  const progressBarWidth = isMobile ? 80 : 18;
  const formattedTime = formatTime(elapsedTime, TOTAL_TIME);
  const props = useSpring({
    width: "100%",
    from: { width: "0%" },
    config: {
      duration: TOTAL_TIME,
    },
    onChange(data) {
      setElapsedTime((parseFloat(data.value.width) / 100) * TOTAL_TIME);
    },
  });

  return (
    <div
      className={`${styles["progress-bar-container"]} flex justify-between items-center sm:pr-4`}
      style={{
        width: `${progressBarWidth}vw`,
        flex: 1,
      }}
    >
      <div
        className={`${styles["progress-bar-inner-container"]} flex justify-between`}
        style={{
          width: `${(value / max) * 0.8 * progressBarWidth}vw`,
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
