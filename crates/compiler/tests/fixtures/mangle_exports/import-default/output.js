//index.js:
 function forEach(arr, fn) {
    for(let i = 0; i < arr.length; i++){
        fn(arr[i]);
    }
}
var for_each_ts_default = forEach;
var simpleCheckForValidColor = function simpleCheckForValidColor(data) {
    var keysToCheck = [
        'r',
        'g',
        'b',
        'a',
        'h',
        's',
        'l',
        'v'
    ];
    var checked = 0;
    var passed = 0;
    for_each_ts_default(keysToCheck, function(letter) {
        if (data[letter]) {
            checked += 1;
            if (!isNaN(data[letter])) {
                passed += 1;
            }
            if (letter === 's' || letter === 'l') {
                var percentPatt = /^\d+%$/;
                if (percentPatt.test(data[letter])) {
                    passed += 1;
                }
            }
        }
    });
    return checked === passed ? data : false;
};
export { simpleCheckForValidColor as simpleCheckForValidColor };
