"use client";
import { cn } from "../../lib/utils";
import React from "react";
import { BentoGrid, BentoGridItem } from "../ui/bento-grid";
import Translate from "@docusaurus/Translate";
import Rocket from "@site/static/img/rocket.png";
import Plug from "@site/static/img/plug.png";
import FeaturePng from "@site/static/img/feature.png";
import Box from "@site/static/img/box.png";
import Compatibility from "@site/static/img/compatible.png";
import Consistency from "@site/static/img/consistency.png";
import {
  IconBoxAlignRightFilled,
  IconClipboardCopy,
  IconFileBroken,
  IconSignature,
  IconTableColumn,
} from "@tabler/icons-react";
import { motion } from "framer-motion";
import { AnimatedBeamPig } from "../ui/beam";

export default function BentoGridCard() {
  return (
    <BentoGrid className="max-w-full mx-auto md:auto-rows-auto relative z-auto">
      {items.map((item, i) => (
        <BentoGridItem
          key={i}
          title={item.title}
          description={item.description}
          header={item.header}
          className={cn("[&>p:text-lg]", item.className)}
          icon={item.icon}
        />
      ))}
    </BentoGrid>
  );
}

const SkeletonOne = () => {
  const variants = {
    initial: {
      x: 0,
      y: 0,
      opacity: 1,
    },
    animate: {
      x: [0, 12],
      transition: {
        duration: .2, // 
        ease: "easeInOut", // 
      },
    },
  };

  return (
    <motion.div
      initial="initial"
      whileHover="animate"
      className="flex flex-1 w-full h-full min-h-[6rem] dark:bg-dot-white/[0.2] bg-dot-black/[0.2] flex-col space-y-2"
    >
      <motion.div
        className="flex justify-center flex-row rounded-full border border-neutral-100 dark:border-white/[0.2] p-2 items-center space-x-2 bg-soft dark:bg-soft"
      >
        <motion.img
          src={Rocket}
          alt=""
          className="w-20"
          variants={variants}
        />
      </motion.div>
    </motion.div>
  );
};
const SkeletonTwo = () => {
  const variants = {
    initial: {
      x: 0,
      y: 0,
      opacity: 1,
    },
    animate: {
      x: [0, 12],
      transition: {
        duration: .2, // 
        ease: "easeInOut", // 
      },
    },
  };

  return (
    <motion.div
      initial="initial"
      whileHover="animate"
      className="flex flex-1 w-full h-full min-h-[6rem] dark:bg-dot-white/[0.2] bg-dot-black/[0.2] flex-col space-y-2"
    >
      <motion.div
        className="flex justify-center flex-row rounded-full border border-neutral-100 dark:border-white/[0.2] p-2 items-center space-x-2 bg-soft dark:bg-soft"
      >
        <motion.img
          src={Box}
          alt=""
          className="w-20"
          variants={variants}
        />
      </motion.div>
    </motion.div>
  );
};
const SkeletonThree = () => {
  const variants = {
    initial: {
      x: 0,
      y: 0,
      opacity: 1,
    },
    animate: {
      x: [0, 12],
      transition: {
        duration: .2, // 
        ease: "easeInOut", // 
      },
    },
  };

  return (
    <motion.div
      initial="initial"
      whileHover="animate"
      className="flex flex-1 w-full h-full min-h-[6rem] dark:bg-dot-white/[0.2] bg-dot-black/[0.2] flex-col space-y-2"
    >
      <motion.div
        className="flex justify-center flex-row rounded-full border border-neutral-100 dark:border-white/[0.2] p-2 items-center space-x-2 bg-soft dark:bg-soft"
      >
        <motion.img
          src={Compatibility}
          alt=""
          className="w-20"
          variants={variants}
        />
      </motion.div>
    </motion.div>
  );
};
const SkeletonFour = () => {
  const variants = {
    initial: {
      x: 0,
      y: 0,
      opacity: 1,
    },
    animate: {
      x: [0, 12],
      transition: {
        duration: .2, // 
        ease: "easeInOut", // 
      },
    },
  };

  return (
    <motion.div
      initial="initial"
      whileHover="animate"
      className="flex flex-1 w-full h-full min-h-[6rem] dark:bg-dot-white/[0.2] bg-dot-black/[0.2] flex-col space-y-2"
    >
      <AnimatedBeamPig />
    </motion.div>
  );
};
const SkeletonFive = () => {
  const variants = {
    initial: {
      x: 0,
      y: 0,
      opacity: 1,
    },
    animate: {
      x: [0, 12],
      transition: {
        duration: .2, // 
        ease: "easeInOut", // 
      },
    },
  };

  return (
    <motion.div
      initial="initial"
      whileHover="animate"
      className="flex flex-1 w-full h-full justify-center min-h-[6rem] dark:bg-dot-white/[0.2] bg-dot-black/[0.2] flex-col space-y-2"
    >
      <motion.div
        className="flex justify-center flex-row rounded-full border border-neutral-100 dark:border-white/[0.2] p-2 items-center space-x-2 bg-soft dark:bg-soft"
      >
        <motion.img
          src={FeaturePng}
          alt=""
          className="w-20"
          variants={variants}
        />
      </motion.div>
    </motion.div>
  );
};
const items = [
  {
    title: <Translate>Extremely Fast</Translate>,
    description: (
      <span className="text-sm">
        <Translate>
          Written in Rust, start a React / Vue project in milliseconds and perform an HMR update within 10ms for most situations.
        </Translate>
      </span>
    ),
    header: <SkeletonOne />,
    className: "md:col-span-1",
    icon: <IconClipboardCopy className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <Translate>Incremental Building</Translate>,
    description: (
      <span className="text-sm">
        <Translate>
          Incremental Building: Support persistent cache, module level cache
          enabled by default, any module won't be compiled twice until it's
          changed!
        </Translate>
      </span>
    ),
    header: <SkeletonTwo />,
    className: "md:col-span-1",
    icon: <IconFileBroken className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <Translate>Partial Bundling</Translate>,
    description: (
      <span className="text-sm">
        <Translate>
          Partial Bundling: Bundle your project into a few reasonable bundles,
          speeding up resource loading without losing caching granularity.
        </Translate>
      </span>
    ),
    header: <SkeletonThree />,
    className: "md:col-span-1",
    icon: <IconSignature className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <Translate>Rich Features and Fully Pluggable</Translate>,
    description: (
      <span className="text-sm">
        <Translate>
          Farm supports compiling HTML, CSS, CSS Modules, Js/Jsx/Ts/Tsx, JSON, Static Assets out of the box, supports Sass, Less, PostCSS, Vue, React, Solid by way of official plugins, supports lazy compiling, partial bundling and more. Everything inside Farm is powered by plugins, Supports both Rust and JavaScript plugins. Support Vite plugins out of box.
        </Translate>
      </span>
    ),
    header: <SkeletonFour />,
    className: "md:col-span-2",
    icon: <IconTableColumn className="h-4 w-4 text-neutral-500" />,
  },

  {
    title: <Translate>Consistency and Compatibility</Translate>,
    description: (
      <span className="text-sm">
        <Translate>
          What you see in development will be the same as what you get in
          production. Supports both legacy (ES5) and modern browsers.
        </Translate>
      </span>
    ),
    header: <SkeletonFive />,
    className: "md:col-span-1",
    icon: <IconBoxAlignRightFilled className="h-4 w-4 text-neutral-500" />,
  },
];
