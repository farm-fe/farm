import { Link, useLocale } from "@farmfe/docs/client";
import { SiteText } from "../site-text";
// import useBaseUrl from '@docusaurus/useBaseUrl';
// import ThemedImage from '@theme/ThemedImage';
import clsx from "clsx";
import React from "react";
import Benchmark from "../components/Benchmark";
import AnimatedGradientStarWithGithub from "../components/MagicUi/animated-shiny-text";
import BlurFade from "../components/MagicUi/blur-fade";
import BentoGridCard from "../components/MagicUi/card";
// import StarrySky from "../components/StarrySky";
// import { AuroraBackground } from "../components/ui/aurora-back";
import styles from "./index.module.css";

function HomepageHeader() {
  const prefix = useLocale() === "zh" ? "/zh/docs/" : "/docs/";
  return (
    <header
      className={clsx(
        "grid grid-cols-1 gap-10 relative z-10 mx-auto max-w-8xl py-4 sm:py-6 lg:py-8",
        "lg:grid-cols-2",
        styles.heroBanner
      )}
    >
      <div className="container w-full flex flex-col my-1 px-2">
        <BlurFade delay={0.25} inView>
          <div className="font-extrabold text-3xl sm:text-6xl lg:text-6xl text-left mb-6 flex flex-col gap-2">
            <div>
              <span>
                <SiteText id="home.extremely" />
              </span>
              <span className={clsx(styles.banner)}>
                <SiteText id="home.fast" />
              </span>
              <span>
                <SiteText id="home.web" />
              </span>
            </div>
            <div>
              <span className={clsx(styles.banner)}>
                <SiteText id="home.buildTool" />
              </span>
            </div>
            <div>
              <span>
                <SiteText id="home.writtenIn" />
              </span>
              <span className={clsx(styles.banner, "my-6")}>
                <SiteText id="home.rust" />
              </span>
            </div>
          </div>
        </BlurFade>

        <BlurFade delay={0.3 * 2} inView>
          <div className="font-semibold brand-color mb-6 text-1xl sm:text-2xl lg:text-xl tracking-wide text-left  flex flex-col gap-2">
            <div>
              <span className={clsx(styles.banner, "font-extrabold")}>
                <SiteText id="home.farm" />
              </span>
              <span className="font-sans">
                <SiteText id="home.description" />
              </span>
            </div>
          </div>
        </BlurFade>
        <div className={clsx(styles.buttons, "my-2")}>
          <Link to={`${prefix}quick-start`} style={{ textDecoration: "none" }}>
            <div
              className={clsx(
                styles.farmButton,
                "flex w-36 sm:w-40 items-center justify-center font-bold"
              )}
            >
              <SiteText id="home.quickStart" />
            </div>
          </Link>
          <Link
            style={{ textDecoration: "none" }}
            to={`${prefix}why-farm`}
          >
            <div
              className={clsx(
                styles.farmButton2,
                "flex w-36 sm:w-40  items-center justify-center font-bold"
              )}
            >
              <SiteText id="home.whyFarm" />
            </div>
          </Link>
        </div>
      </div>
      <Benchmark />
    </header>
  );
}


const HomeBaseContent = () => {
  const mainContent = React.useMemo(() => {
    return (
      <div className="mb-20 my-10 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8max-w-6xl">
        <AnimatedGradientStarWithGithub />
        <HomepageHeader />
        <BentoGridCard />
      </div>
    );
  }, []);

    return (
      <>
        {mainContent}
      </>
    );
};


export const metadata = { title: "Farm v2", description: "Rust-powered web build and compilation architecture" };

export default function Home() {
  const zh = useLocale() === 'zh';
  return <div className="farm-online-home"><aside className="farm-online-v2"><strong>Farm v2 · All in Rust</strong><span>{zh ? '构建与编译架构由 Rust 驱动；本站界面由 React 渲染。' : 'Rust-powered build and compilation architecture; this site UI is rendered with React.'}</span></aside><HomeBaseContent /></div>;
}
