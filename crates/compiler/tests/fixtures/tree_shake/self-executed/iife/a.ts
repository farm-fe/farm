// IIFE

(function () {
  const iife_foo = 'iife-foo';
  // console.log(iife_foo);
})();

// IIFE

var UrlType;
(function (UrlType) {
  UrlType[(UrlType['Empty'] = 1)] = 'Empty';
  UrlType[(UrlType['Hash'] = 2)] = 'Hash';
  UrlType[(UrlType['Query'] = 3)] = 'Query';
  UrlType[(UrlType['RelativePath'] = 4)] = 'RelativePath';
  UrlType[(UrlType['AbsolutePath'] = 5)] = 'AbsolutePath';
  UrlType[(UrlType['SchemeRelative'] = 6)] = 'SchemeRelative';
  UrlType[(UrlType['Absolute'] = 7)] = 'Absolute';
})(UrlType || (UrlType = {}));

export default function () {
  console.log('foo');
}
