function $e() {
  return {
    seed: (e) => {
      return e;
    }
  }
}


var F = class {
  constructor(e = {}){
      let { randomizer: r = $e() } = e;
      this._randomizer = r;
  }
  get defaultRefDate() {
      return this._defaultRefDate;
  }
  setDefaultRefDate(e = ()=>new Date) {
      typeof e == "function" ? this._defaultRefDate = e : this._defaultRefDate = ()=>new Date(e);
  }
  seed(e = Math.ceil(Math.random() * Number.MAX_SAFE_INTEGER)) {
      return this._randomizer.seed(e), e;
  }
}, Yt = new F;

export { F as default, Yt as R };