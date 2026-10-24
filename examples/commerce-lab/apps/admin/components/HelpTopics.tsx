import React from 'react';
export type HelpTopic = { id: string; title: string; keywords: string };
export function HelpTopics({ topics, selected, onSelect }: { topics: HelpTopic[]; selected: string; onSelect: (id: string) => void }) {
  return <nav aria-label="Help topics">{topics.length ? topics.map(topic => <button key={topic.id} aria-pressed={selected === topic.id} onClick={() => onSelect(topic.id)}>{topic.title}</button>) : <p>No guides match your search.</p>}</nav>;
}
