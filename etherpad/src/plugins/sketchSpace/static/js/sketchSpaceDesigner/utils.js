dojo.provide("sketchSpaceDesigner.utils");

sketchSpaceDesigner.utils.setObject = function (dst, src, onlyDefault) {
  for (name in src) {
    if (typeof(src[name]) == "object") {
      if (dst[name] === undefined) dst[name] = {};
      sketchSpaceDesigner.utils.setObject(dst[name], src[name], onlyDefault);
    } else {
      if (!onlyDefault || dst[name] === undefined)
        dst[name] = src[name];
    }
  }
  return dst;
}