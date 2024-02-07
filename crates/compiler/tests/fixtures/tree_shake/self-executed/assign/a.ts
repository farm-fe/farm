import mapCacheClear from './b';

function MapCache(entries) {
  var index = -1,
    length = entries == null ? 0 : entries.length;

  this.clear();
  while (++index < length) {
    var entry = entries[index];
    this.set(entry[0], entry[1]);
  }
}

var a = null;

// Add methods to `MapCache`.
MapCache.prototype.clear = mapCacheClear;
MapCache.prototype.clear = () => (a, mapCacheClear);

export default MapCache;
