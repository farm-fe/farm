import { useEffect, useCallback } from "react";
import * as stylex from "@stylexjs/stylex";
import { Button } from "../components/button";
import { Input } from "../components/input";
import { colors, darkTheme, lightTheme } from "../themes/color.stylex";

type Theme = "light" | "dark" | "auto";

function setTheme(theme: Theme) {
  const isDark =
    theme === "dark" ||
    (theme === "auto" &&
      window.matchMedia?.("(prefers-color-scheme: dark)")?.matches);

  document.documentElement.setAttribute(
    "class",
    stylex.props(isDark ? darkTheme : lightTheme).className,
  );
  localStorage.setItem("visualizer-color-scheme", theme);
}

function toggleTheme() {
  const currentTheme =
    (localStorage.getItem("visualizer-color-scheme") as Theme) || "auto";
  const newTheme = currentTheme === "dark" ? "light" : "dark";
  setTheme(newTheme);
}

export default function Layout() {
  const initTheme = useCallback(() => {
    const savedTheme =
      (localStorage.getItem("visualizer-color-scheme") as Theme) || "auto";
    setTheme(savedTheme);
  }, []);

  useEffect(() => {
    initTheme();
  }, [initTheme]);

  return (
    <>
      <nav
        stylex={{
          height: "54px",
          boxSizing: "border-box",
          padding: "0 6px",
          background: colors.background,
        }}
      >
        <Button onClick={toggleTheme}>切换主题</Button>
      </nav>
    </>
  );
}
