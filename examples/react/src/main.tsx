import React, { useEffect } from "react";
import { useStore } from "react-redux";

import { Welcome } from "./components/index";
import "./main.css";

import { BizType } from "./enums";

import * as Sentry from "@sentry/react";
import { Effect } from "effect";

import styles from "./index.module.scss";
// @ts-ignore
// import { a } from "virtual-module"

Sentry.init({});

const result = Effect.runSync(Effect.succeed(42));

export function Main() {
  const store = useStore();
  console.log(import.meta.env);
  useEffect(() => {

    (async () => {

      const result = await import("virtual-module")
      console.log(result)
    })()
  }, [])
  return (
    <>
      <div style={{ color: "#fff" }} className={styles.main}>
        <div>effect: {result}</div>
        <div style={{ width: "100px", color: "#fff" }}>
          <b>store.api.config.online: </b>
          {JSON.stringify(store.getState().api.config.online)}
          BizType: {BizType.First} {BizType.Second}
        </div>
      </div>
      <Welcome />
    </>
  );
}
