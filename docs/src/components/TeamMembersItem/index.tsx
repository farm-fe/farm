import React from "react";
import clsx from "clsx";
import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import styles from "./index.module.css";

export default function TeamMembersItem({ member }) {
  return (
    <article className={clsx(styles.teamMembersItem)}>
      <div className={clsx(styles.profile)}>
        <figure
          className={styles.avatar}
          onClick={() => window.open(member.orgLink, "_blank")}
        >
          <img
            className={styles["avatar-img"]}
            src={member.avatar}
            alt={member.name}
          />
        </figure>
        <div className={styles.data}>
          <h1 className={styles.name}>{member.name}</h1>
          <h2 className={styles.org}>{member.org}</h2>
          {member.desc ? <p className={styles.desc}>{member.desc}</p> : null}
        </div>
      </div>
    </article>
  );
}
