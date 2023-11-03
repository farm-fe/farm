export class DepNoInlineSources {
    constructor() {
        this.a = 'a';
        const cc = 'c';
        this.a = this.a + cc;
    }
    setA(a) {
        this.a = a;
    }
    getA() {
        return this.a;
    }
}
//# sourceMappingURL=dep-no-inline-sources.js.map