var farm_p_a = require('./a');
var farm_p_b = require('./b');
var farm_p_c = require('./c');
var farm_p = await Promise.all([farm_p_a, farm_p_b, farm_p_c]);
var a = farm_p[0].default;
var b = farm_p[1].default;
var d = farm_p[2].c;
console.log(a, b, c);