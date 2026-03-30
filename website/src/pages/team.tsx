import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import Layout from "@theme/Layout";
import React from "react";
import { TeamMembers } from "../components/TeamMembers";
import { members } from "./data";

export default function Team() {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title} Documentation`}
      description="Description will go into a meta tag in <head />"
    >
      <div className="w-9/12 m-auto">
        <TeamMembers members={members} />
      </div>
    </Layout>
  );
}
