
export class DepNoInlineSources {
  a: string = 'a';

  constructor() {
    type c = string;
    const cc: c = 'c';
    this.a = this.a + cc;  
  }

  setA(a: string) {
    this.a = a;
  }

  getA(): string {
    return this.a;
  }
}