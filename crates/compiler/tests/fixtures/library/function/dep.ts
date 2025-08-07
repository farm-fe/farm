const MIN_CHART_WIDTH = 1;
const MIN_CHART_HEIGHT = 1;

function getElementSize(ele) {
  var style = getComputedStyle(ele);
  return {
    width:
      (ele.clientWidth || parseInt(style.width, 10)) -
      parseInt(style.paddingLeft, 10) -
      parseInt(style.paddingRight, 10),
    height:
      (ele.clientHeight || parseInt(style.height, 10)) -
      parseInt(style.paddingTop, 10) -
      parseInt(style.paddingBottom, 10)
  };
}

function isNumber(v) {
  return typeof v === 'number' && !isNaN(v);
}

export function getChartSize(ele, autoFit, width, height) {
  var w = width;
  var h = height;
  if (autoFit) {
    var size = getElementSize(ele);
    w = size.width ? size.width : w;
    h = size.height ? size.height : h;
  }
  return {
    width: Math.max(isNumber(w) ? w : MIN_CHART_WIDTH, MIN_CHART_WIDTH),
    height: Math.max(isNumber(h) ? h : MIN_CHART_HEIGHT, MIN_CHART_HEIGHT)
  };
}
export function removeDom(dom) {
  var parent = dom.parentNode;
  if (parent) {
    parent.removeChild(dom);
  }
}

export default {
  getChartSize,
  removeDom,
}