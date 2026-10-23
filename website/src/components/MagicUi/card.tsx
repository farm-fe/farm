import { SiteText } from "../../site-text";

"use client";

import {
  IconBoxAlignRightFilled,
  IconClipboardCopy,
  IconFileBroken,
  IconSignature,
  IconTableColumn,
} from "@tabler/icons-react";
import { motion } from "framer-motion";
import React from "react";
import Box from "../../../static/img/box.png";
import Compatibility from "../../../static/img/compatible.png";
import Consistency from "../../../static/img/consistency.png";
import FeaturePng from "../../../static/img/feature.png";
import Plug from "../../../static/img/plug.png";
import Rocket from "../../../static/img/rocket.png";
import { cn } from "../../lib/utils";
import { AnimatedBeamPig } from "../ui/beam";
import { BentoGrid, BentoGridItem } from "../ui/bento-grid";

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
    title: <SiteText id="features.fast.title" />,
    description: (
      <span className="text-sm">
        <SiteText id="features.fast.description" />
      </span>
    ),
    header: <SkeletonOne />,
    className: "md:col-span-1",
    icon: <IconClipboardCopy className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <SiteText id="features.incremental.title" />,
    description: (
      <span className="text-sm">
        <SiteText id="features.incremental.description" />
      </span>
    ),
    header: <SkeletonTwo />,
    className: "md:col-span-1",
    icon: <IconFileBroken className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <SiteText id="features.bundling.title" />,
    description: (
      <span className="text-sm">
        <SiteText id="features.bundling.description" />
      </span>
    ),
    header: <SkeletonThree />,
    className: "md:col-span-1",
    icon: <IconSignature className="h-4 w-4 text-neutral-500" />,
  },
  {
    title: <SiteText id="features.pluggable.title" />,
    description: (
      <span className="text-sm">
        <SiteText id="features.pluggable.description" />
      </span>
    ),
    header: <SkeletonFour />,
    className: "md:col-span-2",
    icon: <IconTableColumn className="h-4 w-4 text-neutral-500" />,
  },

  {
    title: <SiteText id="features.compatibility.title" />,
    description: (
      <span className="text-sm">
        <SiteText id="features.compatibility.description" />
      </span>
    ),
    header: <SkeletonFive />,
    className: "md:col-span-1",
    icon: <IconBoxAlignRightFilled className="h-4 w-4 text-neutral-500" />,
  },
];
