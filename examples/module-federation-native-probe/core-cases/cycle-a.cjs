exports.name='a'; const b=require('./cycle-b.cjs'); exports.peer=b.name;
