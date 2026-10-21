import { Link } from "@farmfe/docs/client";
import { SiteText } from "../site-text";
import { useColorMode, useLocale } from "@farmfe/docs/client";
import clsx from "clsx";
import React from "react";
import Benchmark from "../components/Benchmark";
import AnimatedGradientStarWithGithub from "../components/MagicUi/animated-shiny-text";
import BlurFade from "../components/MagicUi/blur-fade";
import BentoGridCard from "../components/MagicUi/card";
import NeonGradientCard from "../components/MagicUi/neon-gradient-card";
import StarrySky from "../components/StarrySky";
import { AuroraBackground } from "../components/ui/aurora-back";
import styles from "./index.module.css";

const homeLinks = { en: { quick: "/docs/quick-start", why: "/docs/why-farm", contribution: "/docs/contribution" }, zh: { quick: "/zh/docs/quick-start", why: "/zh/docs/why-farm", contribution: "/zh/docs/contribution" } };

export const metadata = { title: "Farm Documentation", description: "Description will go into a meta tag in <head />" };

function HomepageHeader() {
  const links = homeLinks[useLocale() === "zh" ? "zh" : "en"];
  return (
    <header
      className={clsx(
        "grid grid-cols-1 gap-10 relative z-10 mx-auto max-w-8xl py-4 sm:py-6 lg:py-8",
        "lg:grid-cols-3",
        styles.heroBanner,
      )}
    >
      <div className="container w-full flex flex-col my-1 px-2 col-span-2">
        <BlurFade delay={0.25} inView>
          <div className="font-extrabold text-3xl sm:text-6xl lg:text-7xl text-left mb-6 flex flex-col gap-2">
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
          <Link to={links.quick} style={{ textDecoration: "none" }}>
            <div
              className={clsx(
                styles.farmButton,
                "flex w-36 sm:w-40 items-center justify-center font-bold",
              )}
            >
              <SiteText id="home.quickStart" />
            </div>
          </Link>
          <Link
            style={{ textDecoration: "none" }}
            to={links.why}
          >
            <div
              className={clsx(
                styles.farmButton2,
                "flex w-36 sm:w-40  items-center justify-center font-bold",
              )}
            >
              <SiteText id="home.whyFarm" />
            </div>
          </Link>
          <Link to={links.contribution} style={{
            textDecoration: "none"
          }}>
            <div
              className={clsx(
                styles.farmButton,
                "flex w-36 sm:w-40 items-center justify-center font-bold",
              )}
            >
              <SiteText id="home.contribute" />
            </div>
          </Link>
        </div>
      </div>
    </header >
  );
}

const HomeBaseContent = () => {
  const { resolvedColorMode } = useColorMode();

  const mainContent = React.useMemo(() => {
    return (
      <div className="mb-20 my-10 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8max-w-6xl">
        <AnimatedGradientStarWithGithub />
        <HomepageHeader />
        <NeonGradientCard>
        </NeonGradientCard>
        <Benchmark />
        <BentoGridCard />
      </div>
    );
  }, []);

  if (resolvedColorMode === "dark") {
    return (
      <>
        <StarrySky />
        {mainContent}
      </>
    );
  } else {
    return (
      <>
        <AuroraBackground />
        {mainContent}
      </>
    );
  }
};

export default function Home() {
  return <HomeBaseContent />;
}
