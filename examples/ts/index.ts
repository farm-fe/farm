import { Project } from 'ts-morph';

interface obj {
  name: string;
  age: number;
}

export const obj: obj = {
  name: 'erkelost',
  age: 18
};

export const project = new Project();

console.log(project);
