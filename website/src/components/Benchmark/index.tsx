import { SiteText } from "../../site-text";
import { ProgressBar } from "./ProgressBar";
import { useInView } from "react-intersection-observer";
import styles from "./index.module.css";
import { Link, useLocale } from "@farmfe/docs/client";
import React, { useState } from "react";
import clsx from "clsx";
import ShinyTextEx from "../MagicUi/shiny-text";
const BENCHMARK_DATA = {
  ColdStart: [
    {
      name: "Farm",
      time: 0.396,
    },
    {
      name: "RsBuild",
      time: 0.468,
    },
    {
      name: "Vite",
      time: 1.700,
    },
    {
      name: "Webpack",
      time: 2.078,
    },
  ],
  HotStart: [
    {
      name: "Farm",
      time: 0.273,
    },
    {
      name: "RsBuild",
      time: 0.468,
    },
    {
      name: "Vite",
      time: 1.426,
    },
    {
      name: "Webpack",
      time: 0.945,
    },
  ],
  HmrRoot: [
    {
      name: "Farm",
      time: 0.018,
    },
    {
      name: "RsBuild",
      time: 0.087,
    },
    {
      name: "Vite",
      time: 0.022,
    },
    {
      name: "Webpack",
      time: 0.532,
    },
  ],
  HmrLeaf: [
    {
      name: "Farm",
      time: 0.013,
    },
    {
      name: "RsBuild",
      time: 0.074,
    },
    {
      name: "Vite",
      time: 0.011,
    },
    {
      name: "Webpack",
      time: 0.165,
    },
  ],
  ColdBuild: [
    {
      name: "Farm",
      time: 0.313,
    },
    {
      name: "RsBuild",
      time: 0.363,
    },
    {
      name: "Vite",
      time: 1.543,
    },
    {
      name: "Webpack",
      time: 4.128,
    },
  ],
  HotBuild: [
    {
      name: "Farm",
      time: 0.16,
    },
    {
      name: "RsBuild",
      time: 0.363,
    },
    {
      name: "Vite",
      time: 1.540,
    },
    {
      name: "Webpack",
      time: 0.527,
    },
  ],
};
// Stable component identities retain scroll position and keyboard focus on selection.
function Pill({ section, visibleSection, onSelect }) {
    return (
      <div>
        <button
          type="button"
          aria-pressed={visibleSection === section.title}
          className={clsx(
            "flex-1 cursor-pointer rounded-md border-0 py-2 px-2 sm:px-4 text-center font-jakarta text-sm font-semibold",
            visibleSection === section.title
              ? "bg-fuchsia-600 text-white"
              : "color-re bg-transparent"
          )}
          onClick={() => onSelect(section.title)}
        >
          {section.name}
        </button>
      </div>
    );
  }

function PillTabs({ SCENE, children, visibleSection, onSelect }) {
    return (
      <div>
        <div className="max-w-full overflow-x-auto">
          <div role="group" aria-label="Benchmark scenarios" className={clsx(styles.pills, "inline-flex mb-4 items-center rounded-lg text-sm lg:text-base")}>
            {SCENE.map((item, index) => {
              return <Pill section={item} key={index} visibleSection={visibleSection} onSelect={onSelect}></Pill>;
            })}
          </div>
        </div>
        <div>{children}</div>
      </div>
    );
  }
export default function Benchmark() {
  const SCENE = [
    { name: <SiteText id="benchmark.coldStart" />, title: "ColdStart" },
    { name: <SiteText id="benchmark.hotStart" />, title: "HotStart" },
    { name: <SiteText id="benchmark.hmrRoot" />, title: "HmrRoot" },
    { name: <SiteText id="benchmark.hmrLeaf" />, title: "HmrLeaf" },
    { name: <SiteText id="benchmark.coldBuild" />, title: "ColdBuild" },
    { name: <SiteText id="benchmark.hotBuild" />, title: "HotBuild" },
  ];
  const zh = useLocale() === "zh";
  const [activeScene, setActiveScene] = useState("ColdStart");
  const { ref, inView } = useInView();
  const performanceInfoList = BENCHMARK_DATA[activeScene];

  const [visibleSection, setVisibleSection] = useState("ColdStart");

  return (
    <>
      <div ref={ref} className="flex min-w-0 w-full">
        <>
          <div
            className={`${styles.tabs} flex min-w-0 w-full flex-col items-center my-4 z-1`}
          >
            <div className="flex h-20 min-w-0 w-full flex-1 items-center self-start lg:justify-end">
              <div className="min-w-0 w-full">
                <PillTabs SCENE={SCENE} visibleSection={visibleSection} onSelect={section => { setVisibleSection(section); setActiveScene(section); }}>
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
                          <ProgressBar animate={inView}
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

                    <ShinyTextEx />
                  </Link>
                  <p className="text-xs font-normal opacity-70">{zh ? "数据沿用原官网 Benchmark；不是 v2 正式版的新测量。" : "Benchmark data from the original website; not a new v2 release measurement."}</p>
                </div>
              </div>
            </div>
          </div>
        </>
      </div>
    </>
  );
}
