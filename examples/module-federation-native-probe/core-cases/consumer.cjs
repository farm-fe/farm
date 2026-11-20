globalThis.order=['before']; if(globalThis.runAsync) module.exports=require('./async.js').value; globalThis.order.push('after'); if(globalThis.runFailure) require('./bad.js');
