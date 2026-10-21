import clsx from "clsx";
import React from "react";
import { SiteText } from "../../site-text";
import FarmCard from "../Card";
import TeamMembersItem from "../TeamMembersItem";
import styles from "./index.module.css";

export function TeamMembers(props) {
  const { members, size } = props;
  const classes = clsx(styles.teamMembers, "small my-10");
  return (
    <div className={classes}>
      <div
        className={clsx(
          styles.banner,
          "flex justify-center my-8 text-4xl font-bold"
        )}
      >
        <SiteText id="team.introduction" />
      </div>
      <div className={clsx("mt-10", styles.container)}>
        {members.map((member) => (
          <div key={member.name} className="w-full item p-2 h-90">
            <FarmCard>
              <TeamMembersItem member={member} />
            </FarmCard>
          </div>
        ))}
      </div>
    </div>
  );
}
