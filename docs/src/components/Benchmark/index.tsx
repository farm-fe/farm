import { ProgressBar } from "./ProgressBar";
import farmCard from "../Card";
import { useInView } from "react-intersection-observer";
import styles from "./index.module.css";
import Translate from "@docusaurus/Translate";
import Link from "@docusaurus/Link";
import React, { useState } from "react";
import clsx from "clsx";
import ShinyTextEx from "../MagicUi/shiny-text";
const BENCHMARK_DATA = {
  ColdStart: [
    {
      name: "farm",
      time: 0.396,
    },
    {
      name: "rsbuild",
      time: 0.468,
    },
    {
      name: "vite",
      time: 1.700,
    },
    {
      name: "webpack",
      time: 2.078,
    },
  ],
  HotStart: [
    {
      name: "farm",
      time: 0.273,
    },
    {
      name: "rsbuild",
      time: 0.468,
    },
    {
      name: "vite",
      time: 1.426,
    },
    {
      name: "webpack",
      time: 0.945,
    },
  ],
  HmrRoot: [
    {
      name: "farm",
      time: 0.018,
    },
    {
      name: "rsbuild",
      time: 0.087,
    },
    {
      name: "vite",
      time: 0.022,
    },
    {
      name: "webpack",
      time: 0.532,
    },
  ],
  HmrLeaf: [
    {
      name: "farm",
      time: 0.013,
    },
    {
      name: "rsbuild",
      time: 0.074,
    },
    {
      name: "vite",
      time: 0.011,
    },
    {
      name: "webpack",
      time: 0.165,
    },
  ],
  ColdBuild: [
    {
      name: "farm",
      time: 0.313,
    },
    {
      name: "rsbuild",
      time: 0.363,
    },
    {
      name: "vite",
      time: 1.543,
    },
    {
      name: "webpack",
      time: 4.128,
    },
  ],
  HotBuild: [
    {
      name: "farm",
      time: 0.16,
    },
    {
      name: "rsbuild",
      time: 0.363,
    },
    {
      name: "vite",
      time: 1.540,
    },
    {
      name: "webpack",
      time: 0.527,
    },
  ],
};
export default function Benchmark() {
  const SCENE = [
    { name: <Translate>ColdStart</Translate>, title: "ColdStart" },
    { name: <Translate>HotStart</Translate>, title: "HotStart" },
    { name: <Translate>HmrRoot</Translate>, title: "HmrRoot" },
    { name: <Translate>HmrLeaf</Translate>, title: "HmrLeaf" },
    { name: <Translate>ColdBuild</Translate>, title: "ColdBuild" },
    { name: <Translate>HotBuild</Translate>, title: "HotBuild" },
  ];
  const [activeScene, setActiveScene] = useState("ColdStart");
  const { ref, inView } = useInView();
  const performanceInfoList = BENCHMARK_DATA[activeScene];

  const [visibleSection, setVisibleSection] = useState("ColdStart");

  function Pill({ section }) {
    return (
      <div>
        <div
          className={clsx(
            "flex-1 cursor-pointer rounded-md py-2 px-2 sm:px-4 text-center font-jakarta text-sm font-semibold",
            visibleSection === section.title
              ? "bg-fuchsia-600 text-white"
              : "color-re"
          )}
          onClick={() => {
            setVisibleSection(section.title);
            setActiveScene(section.title);
          }}
        >
          {section.name}
        </div>
      </div>
    );
  }

  function PillTabs({ SCENE, children }) {
    return (
      <div className="overflow-x-auto">
        <div className="inline-flex mb-4 items-center rounded-lg text-sm lg:text-base">
          {SCENE.map((item, index) => {
            return <Pill section={item} key={index}></Pill>;
          })}
        </div>
        <div>{children}</div>
      </div>
    );
  }
  return (
    <>
      <div ref={ref} className="flex">
        <>
          <div
            className={`${styles.tabs} flex flex-col items-center my-4 z-1`}
          >
            <div className="flex h-20 w-full flex-1 items-center self-start lg:justify-end">
              <div className="w-full">
                <PillTabs SCENE={SCENE}>
                  {performanceInfoList.map((info) => (
                    <div
                      key={info.name}
                      className="flex flex-center justify-start my-8 flex-col sm:flex-row"
                    >
                      {inView && (
                        <>
                          <div
                            className="flex items-center text-light-500  text-center font-bold"
                            style={{ minWidth: "100px" }}
                          >
                            {info.name}
                          </div>
                          <ProgressBar
                            value={info.time}
                            max={Math.max(
                              ...performanceInfoList.map((info) => info.time)
                            )}
                          />
                        </>
                      )}
                    </div>
                  ))}
                </PillTabs>
                <div className="font-bold cursor-pointer mt-6">
                  <Link
                    rel="stylesheet"
                    href="https://github.com/farm-fe/performance-compare"
                  >
                    {/* <Translate>See benchmark details</Translate> */}
                    <ShinyTextEx />
                  </Link>
                </div>
              </div>
            </div>
          </div>
        </>
      </div>
    </>
  );
}
