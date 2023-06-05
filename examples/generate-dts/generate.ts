interface IExample {
  name: string;
  age: number;
  obj: any;
}

export const example: IExample = {
  name: 'test',
  age: 1,
  obj: {
    a: 1,
    b: 2
  }
};
