import React, { useRef, useState } from "react";
import { useSpring, animated } from "@react-spring/web";
import { useControls } from "leva";
import styled from "./index.module.css";

export default function Card(props) {
  const cardRef = useRef(null);
  const config = useControls({
    // mass: 1,
    // tension: 170,
    // friction: 26,
    // clamp: false,
    // precision: 0.01,
    // velocity: 0,
  });

  const [{ xys }, api] = useSpring(() => ({ xys: [0, 0, 1] }), [config]);

  const handleMouseLeave = () =>
    api.start({
      xys: [0, 0, 1],
    });

  const handleMouseMove = (e) => {
    const rect = cardRef.current.getBoundingClientRect();
    api.start({
      xys: calc(e.clientX / 1.2, e.clientY / 1.2, rect),
    });
  };

  return (
    <div className={styled.cardMain} ref={cardRef}>
      <animated.div
        className={styled.card}
        style={{ transform: xys.to(trans) }}
        onMouseLeave={handleMouseLeave}
        onMouseMove={handleMouseMove}
      >
        {props.children}
      </animated.div>
    </div>
  );
}

const calc = (x, y, rect) => [
  -(y - rect.top - rect.height / 40) / 8,
  (x - rect.left - rect.width / 40) / 8,
  1.4,
];

const trans = (x, y, s) =>
  `perspective(600px) rotateX(${x}deg) rotateY(${y}deg) scale(${s})`;
