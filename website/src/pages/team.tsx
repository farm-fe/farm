import type { PageMetadata } from "@farmfe/docs";
import React from "react";
import { TeamMembers } from "../components/TeamMembers";
import { members } from "./data";

export const metadata = {
  title: "Farm Documentation",
  description: "Description will go into a meta tag in <head />",
} satisfies PageMetadata;

export default function Team() {
  return (
      <div className="w-9/12 m-auto">
        <TeamMembers members={members} />
      </div>
  );
}
