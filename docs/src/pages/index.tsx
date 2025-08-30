import Link from "@docusaurus/Link";
import Translate from "@docusaurus/Translate";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import clsx from "clsx";
import React from "react";
import Benchmark from "../components/Benchmark";
import AnimatedGradientStarWithGithub from "../components/MagicUi/animated-shiny-text";
import BlurFade from "../components/MagicUi/blur-fade";
import BentoGridCard from "../components/MagicUi/card";
import StarrySky from "../components/StarrySky";
import { AuroraBackground } from "../components/ui/aurora-back";
import { useColorMode } from "@docusaurus/theme-common";
import NeonGradientCard from "../components/MagicUi/neon-gradient-card";
import styles from "./index.module.css";

function HomepageHeader() {
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
                <Translate>Extremely</Translate>
              </span>
              <span className={clsx(styles.banner)}>
                <Translate> Fast </Translate>
              </span>
              <span>
                <Translate>Web</Translate>
              </span>
            </div>
            <div>
              <span className={clsx(styles.banner)}>
                <Translate> Build Tool</Translate>
              </span>
            </div>
            <div>
              <span>
                <Translate>Written in</Translate>
              </span>
              <span className={clsx(styles.banner, "my-6")}>
                <Translate> Rust</Translate>
              </span>
            </div>
          </div>
        </BlurFade>

        <BlurFade delay={0.3 * 2} inView>
          <div className="font-semibold brand-color mb-6 text-1xl sm:text-2xl lg:text-xl tracking-wide text-left  flex flex-col gap-2">
            <div>
              <span className={clsx(styles.banner, "font-extrabold")}>
                <Translate>Farm </Translate>
              </span>
              <span className="font-sans">
                <Translate>
                  is a Rust-Based Web Building Engine to Facilitate Your Web
                  Program and JavaScript Library
                </Translate>
              </span>
            </div>
          </div>
        </BlurFade>
        <div className={clsx(styles.buttons, "my-2")}>
          <Link to="/docs/quick-start" style={{ textDecoration: "none" }}>
            <div
              className={clsx(
                styles.farmButton,
                "flex w-36 sm:w-40 items-center justify-center font-bold",
              )}
            >
              <Translate>Quick Start</Translate>
            </div>
          </Link>
          <Link
            style={{ textDecoration: "none" }}
            to="/docs/why-farm"
          >
            <div
              className={clsx(
                styles.farmButton2,
                "flex w-36 sm:w-40  items-center justify-center font-bold",
              )}
            >
              <Translate>Why Farm?</Translate>
            </div>
          </Link>
          <Link to="/docs/contribution" style={{
            textDecoration: "none"
          }}>
            <div
              className={clsx(
                styles.farmButton,
                "flex w-36 sm:w-40 items-center justify-center font-bold",
              )}
            >
              <Translate>Contribute</Translate>
            </div>
          </Link>
        </div>
      </div>
    </header >
  );
}

const HomeBaseContent = () => {
  const { colorMode } = useColorMode();

  const mainContent = React.useMemo(() => {
    return (
      <main className="mb-20 my-10 max-w-7xl mx-auto w-full px-4 sm:px-6 lg:px-8max-w-6xl">
        <AnimatedGradientStarWithGithub />
        <HomepageHeader />
        <NeonGradientCard>
        </NeonGradientCard>
        <Benchmark />
        <BentoGridCard />
      </main>
    );
  }, []);

  if (colorMode === "dark") {
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
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      //@ts-ignore
      title={`${siteConfig.title} Documentation`}
      description="Description will go into a meta tag in <head />"
    >
      <HomeBaseContent />
    </Layout>
  );
}
