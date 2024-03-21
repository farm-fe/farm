class Foo {}

Foo.create = function () {
  return new Validate();
};

class Validate {
  constructor(obj, options) {
    this.obj = obj;
    this.options = options;
    this.globalConfig = BValidate.globalConfig;
  }
}

var BValidate = function (obj, options) {
  return new Validate(obj, Object.assign({ field: 'value' }, options));
};
BValidate.globalConfig = {};
// 全局生效校验信息
BValidate.setGlobalConfig = function (options) {
  BValidate.globalConfig = options || {};
};

export { Foo, BValidate as default };
